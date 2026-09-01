use std::{convert::Infallible, sync::Arc};

use axum::extract::FromRequestParts;
use chrono_tz::Tz;

use crate::AppState;

pub struct Timezone(pub (Tz, bool));

impl<S> FromRequestParts<S> for Timezone
where
    AppState: From<S>,
    S: Send + Sync + Clone,
{
    type Rejection = Infallible;

    #[tracing::instrument(name = "extract_timezone", skip(parts, state))]
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let app_state = Arc::from(state.clone());

        let t = app_state.maxminddb.get_timezone(&parts.headers).await;

        Ok(Timezone(t))
    }
}
