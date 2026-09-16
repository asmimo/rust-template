use axum::extract::FromRequestParts;
use jiff::tz::TimeZone;

use crate::AppState;

pub struct Timezone(pub (TimeZone, bool));

impl FromRequestParts<AppState> for Timezone {
    type Rejection = std::convert::Infallible;

    /// Extracts the client's timezone from the request headers and application state.
    #[tracing::instrument(name = "extract_timezone", skip(parts, state))]
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let timezone = state.maxminddb.get_timezone(&parts.headers).await;

        Ok(Timezone(timezone))
    }
}
