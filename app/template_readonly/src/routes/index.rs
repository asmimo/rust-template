use axum::{extract::State, response::IntoResponse};

use crate::{AppState, app_error, extractors::timezone::Timezone, templates};
use hypertext::prelude::*;

#[tracing::instrument(skip(_state))]
pub async fn index(
    State(_state): State<AppState>,
    Timezone(timezone): Timezone,
) -> Result<impl IntoResponse, app_error::AppError> {
    let t = rsx! {
        <div>"Hello, world!"</div>
        <div>(format!("{timezone:?}"))</div>
    };

    let layout = templates::layout(&t);

    Ok((
        // [(
        //     http::header::CACHE_CONTROL,
        //     "public, max-age=15, stale-while-revalidate=30",
        // )],
        layout.render(),
    )
        .into_response())
}
