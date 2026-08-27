use axum::http::Request;
use tower_governor::{GovernorError, key_extractor::KeyExtractor};
use utils::maxminddb::get_client_ip_header;

#[derive(Debug, Clone)]
pub struct CustomHeaderExtractor;

impl KeyExtractor for CustomHeaderExtractor {
    type Key = String;

    #[tracing::instrument(name = "governor.extract_header_ip", skip(req))]
    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        let header_key = get_client_ip_header();
        req.headers()
            .get(header_key)
            .and_then(|val| val.to_str().ok())
            .map(std::borrow::ToOwned::to_owned)
            .ok_or(GovernorError::UnableToExtractKey)
    }
}
