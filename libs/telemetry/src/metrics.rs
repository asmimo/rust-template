use opentelemetry::{
    global,
    metrics::{Meter, MeterProvider},
};
use opentelemetry_otlp::{MetricExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};

use crate::{TelemetryConfig, config::OtlpProtocol, error::TelemetryResult};

#[derive(Debug)]
pub struct MetricsExporter {
    pub provider: SdkMeterProvider,
    pub meter: Meter,
}

impl MetricsExporter {
    pub fn new(config: &TelemetryConfig) -> TelemetryResult<Self> {
        let export_builder = MetricExporter::builder();
        let exporter = config
            .otlp_protocol
            .as_ref()
            .map(|otlp_protocol| match otlp_protocol {
                OtlpProtocol::Grpc => export_builder.with_tonic().build(),
                _ => export_builder
                    .with_http()
                    .with_protocol(Protocol::HttpBinary)
                    .build(),
            });

        let provider_builder = SdkMeterProvider::builder().with_resource(config.get_resource());
        let provider = if let Some(exporter) = exporter {
            let reader = PeriodicReader::builder(exporter?).build();
            provider_builder.with_reader(reader).build()
        } else {
            provider_builder.build()
        };

        global::set_meter_provider(provider.clone());

        let meter = provider.meter(config.service_name.clone().leak());

        Ok(Self { provider, meter })
    }

    pub fn force_flush(&self) -> TelemetryResult<()> {
        self.provider.force_flush()?;
        Ok(())
    }
}
