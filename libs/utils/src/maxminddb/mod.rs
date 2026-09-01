use std::{net::IpAddr, sync::OnceLock};

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

#[cfg(debug_assertions)]
static LOCAL_IP: OnceLock<Option<String>> = OnceLock::new();

static MAXMINDDB: OnceCell<MaxmindDbResult<Reader<Vec<u8>>>> = OnceCell::const_new();
pub static CLIENT_IP_HEADER: OnceLock<String> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct MaxMindDB {
    #[cfg_attr(debug_assertions, allow(unused))]
    client_ip_header: &'static String,
    client_timezone_header: Option<String>,
    default_timezone: Tz,
}

impl MaxMindDB {
    pub fn init() -> Self {
        Self {
            client_ip_header: get_client_ip_header(),
            client_timezone_header: env::get_env("CLIENT_TIMEZONE_HEADER").ok(),
            default_timezone: UTC,
        }
    }

    pub fn with_default_timezone(mut self, timezone: impl Into<String>) -> MaxmindDbResult<Self> {
        self.default_timezone = timezone.into().parse()?;

        Ok(self)
    }

    #[tracing::instrument(skip(ip))]
    pub async fn get_city<'a>(ip: IpAddr) -> MaxmindDbResult<City<'a>> {
        if let Ok(reader) = init_maxminddb().await {
            if let Ok(result) = reader.lookup(ip)
                && let Ok(Some(city)) = result.decode::<City<'a>>()
            {
                Ok(city)
            } else {
                Err(MaxmindDbError::Custom("Failed to get reader".to_string()))
            }
        } else {
            Err(MaxmindDbError::Custom("Failed to get reader".to_string()))
        }
    }

    #[cfg(feature = "maxminddb-axum")]
    #[tracing::instrument(skip(self, headers))]
    pub fn get_ip(
        &self,
        #[cfg_attr(debug_assertions, allow(unused))] headers: &axum::http::HeaderMap,
    ) -> Option<IpAddr> {
        #[cfg(debug_assertions)]
        {
            get_local_ip().and_then(|ip| ip.parse().ok())
        }

        #[cfg(not(debug_assertions))]
        {
            Self::get_header_value(headers, &self.client_ip_header).and_then(|ip| ip.parse().ok())
        }
    }

    #[cfg(feature = "maxminddb-axum")]
    #[tracing::instrument(skip(self, headers))]
    pub async fn get_timezone(&self, headers: &axum::http::HeaderMap) -> (Tz, bool) {
        tracing::debug!("Headers: {headers:?}");
        let mut is_fallback = false;
        let timezone = if let Some(client_timezone_header) = &self.client_timezone_header
            && let Some(timezone) = Self::get_header_value(headers, client_timezone_header)
        {
            tracing::debug!(
                "Found through timezone header -> {client_timezone_header}: {timezone}"
            );
            Some(timezone)
        } else if let Some(ip) = self.get_ip(headers)
            && let Some(city) = Self::get_city(ip).await.ok()
            && let Some(timezone) = city.location.time_zone
        {
            tracing::debug!("Found through ip header -> {ip}: {timezone}");
            Some(timezone.to_string())
        } else {
            tracing::debug!(
                "Timezone not found: Using default timezone: {}",
                self.default_timezone
            );
            is_fallback = true;
            None
        };

        let timezone = timezone
            .and_then(|t| t.parse().ok())
            .unwrap_or(self.default_timezone);

        (timezone, is_fallback)
    }

    #[cfg(feature = "maxminddb-axum")]
    pub fn get_header_value(
        headers: &axum::http::HeaderMap,
        header_string: &str,
    ) -> Option<String> {
        let header_names = header_string.split(',').collect::<Vec<&str>>();

        header_names.iter().find_map(|header_name| {
            headers.get(*header_name).and_then(|value| {
                value.to_str().ok().and_then(|value| {
                    value
                        .split(',')
                        .next()
                        .map(|value| value.trim().to_string())
                })
            })
        })
    }
}

#[cfg(debug_assertions)]
#[tracing::instrument]
fn get_local_ip() -> Option<&'static String> {
    LOCAL_IP
        .get_or_init(|| {
            ip_discovery::blocking::get_ipv4()
                .map_err(|err| {
                    println!("Failed to get public ipv4: {err:?}");
                    err
                })
                .ok()
                .and_then(|ips| ips.ipv4().map(|ip| ip.to_string()))
        })
        .as_ref()
}

#[tracing::instrument]
async fn init_maxminddb() -> &'static MaxmindDbResult<Reader<Vec<u8>>> {
    MAXMINDDB
        .get_or_init(|| async {
            let target_file_name = "GeoLite2-City.mmdb";

            if !std::path::Path::new(target_file_name).exists() {
                let url = env::get_env("MAXMINDDB_DOWNLOAD_URL")?;
                let response = reqwest::get(url).await?;
                let bytes = response.bytes().await?;

                let tar = GzDecoder::new(&bytes[..]);
                let mut archive = Archive::new(tar);

                for entry_result in archive.entries()? {
                    let mut entry = entry_result?;

                    let path = entry
                        .path()?
                        .to_str()
                        .ok_or(MaxmindDbError::Custom(
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
        })
        .await
}

pub fn get_client_ip_header() -> &'static String {
    CLIENT_IP_HEADER
        .get_or_init(|| env::get_env_or_default("CLIENT_IP_HEADER", "x-forwarded-for".to_string()))
}
