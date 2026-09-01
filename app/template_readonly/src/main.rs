mod app_error;
mod rate_limiter_extractor;
mod templates;
mod tracing_telemetry;

use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{Router, body::Body, extract::State, http, response::IntoResponse, routing, serve};
use hitbox::{
    Config, Neutral,
    concurrency::BroadcastConcurrencyManager,
    policy::{PolicyConfig, StalePolicy},
};
use hitbox_backend::format::BincodeFormat;
use hitbox_fn::prelude::*;
use hitbox_http::{
    CacheableHttpResponse, extractors::Method as MethodExtractor,
    predicates::request::Method as RequestMethod,
};
use hitbox_moka::MokaBackend;
use hitbox_tower::Cache;
use hypertext::prelude::*;
use reqwest::{Client, ClientBuilder};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{compression::CompressionLayer, services::ServeDir, timeout::TimeoutLayer, trace};
use utils::{env, lettre::Lettre, maxminddb::MaxMindDB};

#[allow(dead_code)]
pub struct AppStateInner {
    pool: PgPool,
    http_client: Client,
    maxminddb: MaxMindDB,
    lettre: Option<Lettre>,
}

type AppState = Arc<AppStateInner>;

impl AppStateInner {
    pub async fn init() -> Result<Self, app_error::AppError> {
        let db_url = env::get_env("DATABASE_URL")?;

        let (pool, http_client, maxminddb, lettre) = tokio::join!(
            PgPoolOptions::new()
                // .max_connections(16)
                .acquire_timeout(std::time::Duration::from_secs(30))
                .idle_timeout(std::time::Duration::from_secs(600))
                .test_before_acquire(true)
                .connect(&db_url),
            async { ClientBuilder::new().build() },
            async { MaxMindDB::init() },
            async { Lettre::init() }
        );

        let lettre = lettre
            .map_err(|err| {
                tracing::warn!("Skipping lettre(Failed): {}", err);
            })
            .ok();

        Ok(Self {
            pool: pool?,
            http_client: http_client?,
            maxminddb,
            lettre,
        })
    }
}

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), app_error::AppError> {
    dotenvy::from_filename(".env").ok();

    let service_name = utils::env::get_env("OTEL_SERVICE_NAME")?;
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

    let cache_backend = MokaBackend::builder()
        .max_entries(10_000)
        .value_format(BincodeFormat)
        .build();
    let policy_config = PolicyConfig::builder()
        .ttl(Duration::from_secs(30))
        .stale(Duration::from_secs(60))
        .stale_policy(StalePolicy::OffloadRevalidate)
        .build();
    let concurrency_manager: BroadcastConcurrencyManager<_> =
        BroadcastConcurrencyManager::<Result<CacheableHttpResponse<Body>, Infallible>>::new();
    let config = Config::builder()
        .request_predicate(RequestMethod::new(http::Method::GET).unwrap())
        .response_predicate(Neutral::new())
        .extractor(MethodExtractor::new())
        .policy(policy_config)
        .build();
    let cache = Cache::builder()
        .backend(cache_backend)
        .config(config)
        .concurrency_manager(concurrency_manager)
        .build();

    tracing::info!("Configuring server");
    let mut app = Router::new()
        // .route("/", routing::get(index))
        .route("/", routing::get(index).layer(cache));

    let limiter = GovernorConfigBuilder::default()
        .const_period(std::time::Duration::from_millis(500))
        .const_burst_size(8)
        .key_extractor(rate_limiter_extractor::CustomHeaderExtractor)
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

#[tracing::instrument(skip(state))]
async fn index(
    State(state): State<AppState>,
    headers: http::HeaderMap,
) -> Result<impl IntoResponse, app_error::AppError> {
    let maxmind = state.maxminddb.get_timezone(&headers).await;
    let ip = format!("{maxmind:?}");

    let t = rsx! {
        <div>"Hello, world!"</div>
        <div>(ip)</div>
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

#[cached]
async fn add(x: i32, y: i32) -> i32 {
    return x + y;
}
