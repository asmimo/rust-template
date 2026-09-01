use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_otlp::{Protocol, SpanExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::trace::{SdkTracerProvider, Tracer};

use crate::{TelemetryConfig, config::OtlpProtocol, error::TelemetryResult};

#[derive(Debug)]
pub struct TracingExporter {
    pub provider: SdkTracerProvider,
    pub tracer: Tracer,
}

impl TracingExporter {
    pub fn new(config: &TelemetryConfig) -> TelemetryResult<Self> {
        let export_builder = SpanExporter::builder();
        let exporter = config
            .otlp_protocol
            .as_ref()
            .map(|otlp_protocol| match otlp_protocol {
                OtlpProtocol::Grpc => export_builder
                    .with_tonic()
                    .with_tls_config(TelemetryConfig::get_tls_config())
                    .build(),
                _ => export_builder
                    .with_http()
                    .with_protocol(Protocol::HttpBinary)
                    .build(),
            });

        let provider_builder = SdkTracerProvider::builder().with_resource(config.get_resource());
        let provider = if let Some(exporter) = exporter {
            provider_builder.with_batch_exporter(exporter?).build()
        } else {
            provider_builder.build()
        };

        global::set_tracer_provider(provider.clone());

        let tracer = provider.tracer(config.service_name.clone());

        Ok(Self { provider, tracer })
    }
}
