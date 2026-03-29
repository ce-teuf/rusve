mod proto;
mod migrations;
mod grpc;
mod profile_service;
mod profile_validation;
mod profile_db;
mod stripe_service;
mod stripe_db;
mod token_db;
mod user_service;
mod user_db;

use crate::proto::users_service_server::UsersServiceServer;
use anyhow::{Context, Result};
use opentelemetry::trace::TracerProvider as _;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct MyService {
    env: service_users::Env,
    pool: deadpool_postgres::Pool,
    metrics: service_users::Metrics,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider().install_default().unwrap();
    // Initalize environment variables
    let env: service_users::Env = service_users::init_envs()?;

    // Initialize tracing + OpenTelemetry
    let meter_provider = service_users::init_metrics("service-users");
    let tracer_provider = service_users::init_tracer("service-users");
    let tracer = tracer_provider.tracer("service-users");
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&env.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    // Connect to database
    let pool = service_users::connect_to_db(&env).context("Failed to connect to database")?;
    tracing::info!("Connected to database");

    // Run migrations
    migrations::run_migrations(&pool)
        .await
        .context("Failed to run migrations")?;
    tracing::info!("Migrations complete");

    // Run gRPC server
    let addr = format!("[::]:{}", env.port).parse()?;
    tracing::info!("gRPC server started on port: {:?}", env.port);
    let server = MyService { env, pool, metrics: service_users::Metrics::new("service-users") };
    let svc = UsersServiceServer::new(server);
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .context("Failed to run gRPC server")?;
    tracer_provider.shutdown()?;
    meter_provider.shutdown()?;
    Ok(())
}
