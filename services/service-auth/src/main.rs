mod auth_db;
mod auth_oauth;
mod auth_service;
mod migrations;
mod proto;

use anyhow::Context;
use anyhow::Result;
use axum::http::StatusCode;
use axum::Json;
use axum::{routing::get, Router};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use http::HeaderValue;
use http::Method;
use opentelemetry::trace::TracerProvider as _;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    env: service_auth::Env,
    pool: deadpool_postgres::Pool,
    metrics: service_auth::Metrics,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider().install_default().unwrap();
    // Initalize environment variables
    let env: service_auth::Env = service_auth::init_envs()?;

    // Initialize tracing + OpenTelemetry
    let meter_provider = service_auth::init_metrics("service-auth");
    let tracer_provider = service_auth::init_tracer("service-auth");
    let tracer = tracer_provider.tracer("service-auth");
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&env.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    // Connect to database
    let pool = service_auth::connect_to_db(&env).context("Failed to connect to database")?;
    tracing::info!("Connected to database");

    // Run migrations
    migrations::run_migrations(&pool)
        .await
        .context("Failed to run migrations")?;
    tracing::info!("Migrations complete");

    // Create shared state
    let shared_state = Arc::new(AppState {
        metrics: service_auth::Metrics::new("service-auth"),
        pool,
        env: env.clone(),
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_origin(env.client_url.parse::<HeaderValue>()?);

    let app = Router::new()
        .route("/", get(root))
        .route("/oauth-login/:provider", get(auth_service::oauth_login))
        .route(
            "/oauth-callback/:provider",
            get(auth_service::oauth_callback),
        )
        .with_state(shared_state.clone())
        .layer(ServiceBuilder::new().layer(cors));

    // Run HTTP server
    let addr = format!("[::]:{}", env.port);
    tracing::info!("HTTP server started on port: {:?}", env.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .await
        .context("Failed to run HTTP server")?;
    tracer_provider.shutdown()?;
    meter_provider.shutdown()?;
    Ok(())
}

/**
 * Ping the database to check if it's up
 */
async fn root() -> Result<(StatusCode, Json<String>), StatusCode> {
    tracing::info!("Ping");
    Ok((StatusCode::OK, Json("Hello, World!".to_string())))
}
