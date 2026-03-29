mod grpc;
mod file_service;
mod file_utils;
mod file_db;
mod email_service;
mod email_validation;
mod email_db;
mod migrations;
mod proto;

use crate::proto::utils_service_server::UtilsServiceServer;
use anyhow::{Context, Result};
use opentelemetry::trace::TracerProvider as _;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct MyService {
    env: service_utils::Env,
    pool: deadpool_postgres::Pool,
    metrics: service_utils::Metrics,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider().install_default().unwrap();
    // Initalize environment variables
    let env: service_utils::Env = service_utils::init_envs()?;

    // Initialize tracing + OpenTelemetry
    let meter_provider = service_utils::init_metrics("service-utils");
    let tracer_provider = service_utils::init_tracer("service-utils");
    let tracer = tracer_provider.tracer("service-utils");
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&env.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    // Connect to database
    let pool = service_utils::connect_to_db(&env).context("Failed to connect to database")?;
    tracing::info!("Connected to database");

    // Run migrations
    migrations::run_migrations(&pool)
        .await
        .context("Failed to run migrations")?;
    tracing::info!("Migrations complete");

    // Run gRPC server
    let addr = format!("[::]:{}", env.port).parse()?;
    tracing::info!("gRPC server started on port: {:?}", env.port);
    let server = MyService { env, pool, metrics: service_utils::Metrics::new("service-utils") };
    let svc = UtilsServiceServer::new(server);
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .context("Failed to run gRPC server")?;
    tracer_provider.shutdown()?;
    meter_provider.shutdown()?;
    Ok(())
}
