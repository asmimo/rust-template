mod app_error;
mod extractors;
mod routes;
mod templates;
mod tracing_telemetry;

use std::sync::Arc;

use axum::{Router, body::Body, http, response::IntoResponse, routing, serve};
use reqwest::{Client, ClientBuilder};
use tower::ServiceExt;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{compression::CompressionLayer, services::ServeDir, timeout::TimeoutLayer, trace};
use utils::{env, lettre::Lettre, maxminddb::MaxMindDB};

use crate::extractors::rate_limiter;

#[allow(dead_code)]
pub struct AppStateInner {
    http_client: Client,
    maxminddb: MaxMindDB,
    lettre: Option<Lettre>,
}

type AppState = Arc<AppStateInner>;

impl AppStateInner {
    pub async fn init() -> Result<Self, app_error::AppError> {
        let (http_client, maxminddb, lettre) = tokio::join!(
            async { ClientBuilder::new().build() },
            async { MaxMindDB::init() },
            async { Lettre::init() }
        );

        let lettre = lettre
            .inspect_err(|err| {
                tracing::warn!("Skipping lettre(Failed): {}", err);
            })
            .ok();

        Ok(Self {
            http_client: http_client?,
            maxminddb,
            lettre,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), app_error::AppError> {
    dotenvy::from_filename(".env").ok();

    let service_name =
        utils::env::get_env_or_default("OTEL_SERVICE_NAME", env!("CARGO_PKG_NAME").to_string());
    tracing_telemetry::init_tracing_with_opentelemetry_subscriber(service_name)
        .expect("Failed to set tracing subscriber with opentelemetry");

    tracing::info!("Initializing application state");
    let state = Arc::new(AppStateInner::init().await?);

    let tracing_layer = trace::TraceLayer::new_for_http()
        .make_span_with(|request: &http::Request<Body>| {
            tracing::info_span!(
                "HTTP request",
                "http.method" = %request.method(),
                "http.route" = %request.uri().path(),
                "http.url" = %request.uri(),
                "http.version" = ?request.version(),
                "otel.name" = format!("{} {}", request.method(), request.uri().path()),
            )
        })
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO))
        .on_failure(trace::DefaultOnFailure::new().level(tracing::Level::ERROR));

    tracing::info!("Configuring server");
    let mut app = Router::new()
        // .route("/", routing::get(index))
        .route("/", routing::get(routes::index));

    let limiter = GovernorConfigBuilder::default()
        .const_period(std::time::Duration::from_millis(500))
        .const_burst_size(8)
        .key_extractor(rate_limiter::CustomHeaderExtractor)
        .finish();

    if let Some(limiter) = limiter {
        let layer = GovernorLayer::new(limiter);
        app = app.layer(layer);
    }

    let timeout_layer = TimeoutLayer::with_status_code(
        http::StatusCode::REQUEST_TIMEOUT,
        std::time::Duration::from_secs(15),
    );

    let compression_layer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);

    let app = app
        .route("/health", routing::get(health_check))
        .layer(tracing_layer)
        // static files
        .nest_service("/assets", routing::get(serve_static_assets))
        .layer(compression_layer)
        .layer(timeout_layer)
        // app state
        .with_state(state);

    let port = env::get_env("PORT")?;
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Server stared on port: {port}");
    serve(listener, app.into_make_service()).await.unwrap();

    Ok(())
}

async fn serve_static_assets(request: http::Request<Body>) -> impl IntoResponse {
    let service = ServeDir::new("./dist");
    let result = service.oneshot(request).await;

    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::header::CACHE_CONTROL,
        http::HeaderValue::from_static("public, max-age=31536000, s-maxage=31536000, immutable"),
    );

    (headers, result)
}

#[tracing::instrument]
async fn health_check(request: http::Request<Body>) -> impl IntoResponse {
    http::StatusCode::OK
}
