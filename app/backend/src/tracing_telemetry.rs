use telemetry::{
    TelemetryConfig, init_telemetry,
    reexports::{
        tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer},
        tracing_subscriber::{EnvFilter, Layer, Registry, fmt, layer::SubscriberExt},
    },
};

pub fn init_tracing_with_opentelemetry_subscriber(
    service_name: String,
) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    let service_version = env!("CARGO_PKG_VERSION");

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = fmt::layer().with_filter(filter.clone());
    let subscriber = Registry::default().with(fmt_layer);

    let enable_metrics = utils::env::get_env_parsed("OTEL_ENABLE_METRICS").unwrap_or(false);
    let enable_tracing = utils::env::get_env_parsed("OTEL_ENABLE_TRACING").unwrap_or(false);

    let config = TelemetryConfig::new(service_name)
        .with_service_version(service_version)
        .with_metrics(enable_metrics)
        .with_tracing(enable_tracing);

    let telemetry = match init_telemetry(&config) {
        Ok(telemetry) => telemetry,
        Err(err) => {
            println!(
                "Failed to initialize tracing with OpenTelemetry: fallback to default tracing\n{err}",
            );
            return tracing::subscriber::set_global_default(subscriber);
        }
    };

    match (telemetry.metrics, telemetry.tracing) {
        (Some(metrics), Some(tracing)) => {
            let metrics_layer = MetricsLayer::new(metrics.provider);
            let telemetry_layer = OpenTelemetryLayer::new(tracing.tracer).with_filter(filter);

            println!("Opentelemetry enabled: metrics and tracing");
            tracing::subscriber::set_global_default(
                subscriber.with(metrics_layer).with(telemetry_layer),
            )
        }
        (Some(metrics), None) => {
            let metrics_layer = MetricsLayer::new(metrics.provider);

            println!("Opentelemetry enabled: metrics");
            tracing::subscriber::set_global_default(subscriber.with(metrics_layer))
        }
        (None, Some(tracing)) => {
            let telemetry_layer = OpenTelemetryLayer::new(tracing.tracer).with_filter(filter);

            println!("Opentelemetry enabled: tracing");
            tracing::subscriber::set_global_default(subscriber.with(telemetry_layer))
        }
        (None, None) => {
            println!("OpenTelemetry not enabled: fallback to default tracing");
            tracing::subscriber::set_global_default(subscriber)
        }
    }
}
