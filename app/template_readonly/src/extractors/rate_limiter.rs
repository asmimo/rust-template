use axum::http::Request;
use tower_governor::{GovernorError, key_extractor::KeyExtractor};
use utils::maxminddb::{MaxMindDB, get_client_ip_header};

#[derive(Debug, Clone)]
pub struct CustomHeaderExtractor;

impl KeyExtractor for CustomHeaderExtractor {
    type Key = String;

    #[tracing::instrument(name = "governor.extract_header_ip", skip(req))]
    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        MaxMindDB::get_header_value(req.headers(), get_client_ip_header())
            .ok_or(GovernorError::UnableToExtractKey)
    }
}
