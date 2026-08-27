use std::{
    net::IpAddr,
    sync::{Arc, OnceLock},
};

use chrono_tz::{Tz, UTC};
use flate2::read::GzDecoder;
use maxminddb::{Reader, geoip2::City};
use tar::Archive;
use tokio::sync::OnceCell;

use crate::{
    env,
    maxminddb::error::{MaxmindDbError, MaxmindDbResult},
};

pub mod error;

static LOCAL_IP: OnceCell<Option<String>> = OnceCell::const_new();
pub static CLIENT_IP_HEADER: OnceLock<String> = OnceLock::new();

pub fn get_client_ip_header() -> &'static String {
    CLIENT_IP_HEADER
        .get_or_init(|| env::get_env_or_default("CLIENT_IP_HEADER", "X-Forwarded-For".to_string()))
}

#[cfg(debug_assertions)]
#[tracing::instrument]
async fn get_local_ip() -> Option<&'static String> {
    LOCAL_IP
        .get_or_init(|| async {
            ip_discovery::get_ipv4()
                .await
                .map_err(|err| {
                    println!("Failed to get public ipv4: {err:?}");
                    err
                })
                .ok()
                .and_then(|ips| ips.ipv4().map(|ip| ip.to_string()))
        })
        .await
        .as_ref()
}

#[derive(Debug, Clone)]
pub struct MaxMindDB {
    reader: Option<Arc<Reader<Vec<u8>>>>,
    #[cfg_attr(debug_assertions, allow(dead_code))]
    client_ip_header: &'static String,
    client_timezone_header: Option<String>,
    default_timezone: String,
}

impl MaxMindDB {
    pub fn init() -> Self {
        Self {
            reader: None,
            client_ip_header: get_client_ip_header(),
            client_timezone_header: None,
            default_timezone: "UTC".to_string(),
        }
    }

    #[must_use]
    pub fn with_client_timezone_header(mut self, header: impl Into<String>) -> Self {
        self.client_timezone_header = Some(header.into());

        self
    }

    #[must_use]
    pub fn with_default_timezone(mut self, timezone: impl Into<String>) -> Self {
        self.default_timezone = timezone.into();

        self
    }

    pub async fn init_old(url: Option<String>) -> MaxmindDbResult<Self> {
        let url = match url {
            Some(url) => url,
            None => env::get_env("MAXMINDDB_DOWNLOAD_URL")?,
        };

        let reader = Self::get_maxminddb_reader(&url).await?;

        let client_timezone_header = env::get_env("CLIENT_TIMEZONE_HEADER").ok();

        let default_timezone = env::get_env_or_default("TIMEZONE", "UTC".to_string());

        Ok(Self {
            reader: Some(Arc::new(reader)),
            client_ip_header: get_client_ip_header(),
            client_timezone_header,
            default_timezone,
        })
    }

    #[tracing::instrument(skip(self, headers_map))]
    pub async fn get_client_timezone<S: ::std::hash::BuildHasher>(
        &self,
        headers_map: &std::collections::HashMap<String, String, S>,
    ) -> MaxmindDbResult<(Tz, Option<String>)> {
        let mut time_zone = headers_map.get("cf-timezone").map(String::as_str);

        #[cfg(debug_assertions)]
        let ip = get_local_ip().await.cloned();

        #[cfg(not(debug_assertions))]
        let ip = {
            let client_ip_headers = self.client_ip_header.split(',').collect::<Vec<&str>>();

            client_ip_headers.iter().find_map(|header_key| {
                headers_map
                    .get(*header_key)
                    .and_then(|ip| ip.split(',').next().map(|ip| ip.trim().to_string()))
            })
        };

        if time_zone.is_none()
            && let Some(client_ip) = ip.as_ref()
            && let Ok(parsed_ip) = client_ip.parse::<IpAddr>()
            && let Some(reader) = &self.reader
            && let Ok(Some(city)) = reader.as_ref().lookup::<City>(parsed_ip)
            && let Some(location) = city.location
            && let Some(tz) = location.time_zone
        {
            time_zone = Some(tz);
        }

        Ok((
            time_zone.map_or(UTC, |tz| tz.parse::<Tz>().unwrap_or(UTC)),
            ip,
        ))
    }

    async fn get_maxminddb_reader(url: &str) -> MaxmindDbResult<Reader<Vec<u8>>> {
        let target_file_name = "GeoLite2-City.mmdb";

        if !std::path::Path::new(target_file_name).exists() {
            let response = reqwest::get(url).await?;
            let bytes = response.bytes().await?;

            let tar = GzDecoder::new(&bytes[..]);
            let mut archive = Archive::new(tar);

            for entry_result in archive.entries()? {
                let mut entry = entry_result?;

                let path = entry
                    .path()?
                    .to_str()
                    .ok_or(MaxmindDbError::Optional(
                        "Failed to convert path to string".to_string(),
                    ))?
                    .to_string();

                if std::path::Path::new(&path)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("mmdb"))
                {
                    entry.unpack(target_file_name)?;
                }
            }
        }

        let reader = Reader::open_readfile(format!("./{target_file_name}"))?;

        Ok(reader)
    }
}
