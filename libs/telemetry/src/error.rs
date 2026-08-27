use thiserror::Error;

#[derive(Error, Debug)]
pub enum TelemetryError {
    #[error(transparent)]
    ExportBuilder(#[from] opentelemetry_otlp::ExporterBuildError),

    #[error(transparent)]
    OTelSdkError(#[from] opentelemetry_sdk::error::OTelSdkError),
}

pub type TelemetryResult<T> = std::result::Result<T, TelemetryError>;
