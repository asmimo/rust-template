use std::str::FromStr;

use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;

use utils::env;

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub service_version: String,
    pub otlp_protocol: Option<OtlpProtocol>,
    pub enable_metrics: bool,
    pub enable_tracing: bool,
}

#[derive(Debug, Clone)]
pub enum OtlpProtocol {
    Grpc,
    HttpProtobuf,
    HttpJson,
}

impl FromStr for OtlpProtocol {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grpc" => Ok(OtlpProtocol::Grpc),
            "http/protobuf" => Ok(OtlpProtocol::HttpProtobuf),
            "http/json" => Ok(OtlpProtocol::HttpJson),
            _ => Err(()),
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        let otlp_protocol_key = "OTEL_EXPORTER_OTLP_PROTOCOL";
        let otlp_protocol = env::get_env(otlp_protocol_key)
            .and_then(|protocol| {
                protocol.parse::<OtlpProtocol>().map_err(|()| {
                    let error = env::EnvError::ParseFailed {
                        key: otlp_protocol_key.to_string(),
                        value: protocol,
                        error: "'grpc', 'http/protobuf' and 'http/json'".to_string(),
                    };

                    println!("{error}");

                    error
                })
            })
            .ok();

        Self {
            service_name: "unknown".to_string(),
            service_version: "0.1.0".to_string(),
            otlp_protocol,
            enable_metrics: false,
            enable_tracing: false,
        }
    }
}

impl TelemetryConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            service_name: name.into(),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_service_version(mut self, value: impl Into<String>) -> Self {
        self.service_version = value.into();

        self
    }

    #[must_use]
    pub fn with_metrics(mut self, value: bool) -> Self {
        self.enable_metrics = value;

        self
    }

    #[must_use]
    pub fn with_tracing(mut self, value: bool) -> Self {
        self.enable_tracing = value;

        self
    }

    pub fn get_resource(&self) -> Resource {
        Resource::builder()
            .with_attribute(KeyValue::new(
                "service.version",
                self.service_version.clone(),
            ))
            .build()
    }
}
