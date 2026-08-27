pub mod config;
pub mod error;
pub mod metrics;
pub mod tracing;

pub use config::TelemetryConfig;
pub use error::{TelemetryError, TelemetryResult};

#[derive(Debug)]
pub struct TelemetryGuard {
    pub metrics: Option<metrics::MetricsExporter>,
    pub tracing: Option<tracing::TracingExporter>,
}

pub fn init_telemetry(config: &TelemetryConfig) -> TelemetryResult<TelemetryGuard> {
    let mut metrics_exporter = None;
    let mut tracing_exporter = None;

    if config.enable_metrics {
        metrics_exporter = Some(metrics::MetricsExporter::new(config)?);
    }

    if config.enable_tracing {
        tracing_exporter = Some(tracing::TracingExporter::new(config)?);
    }

    Ok(TelemetryGuard {
        metrics: metrics_exporter,
        tracing: tracing_exporter,
    })
}

pub mod reexports {
    pub use opentelemetry;
    pub use opentelemetry_sdk;
    pub use opentelemetry_semantic_conventions as semcov;
    pub use tracing_opentelemetry;
    pub use tracing_subscriber;
}
