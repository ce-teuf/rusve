mod migrations;
mod note_db;
mod note_service;
mod note_validation;
mod proto;

use crate::proto::notes_service_server::NotesServiceServer;
use anyhow::{Context, Result};
use opentelemetry::trace::TracerProvider as _;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct MyService {
    env: service_notes::Env,
    pool: deadpool_postgres::Pool,
    metrics: service_notes::Metrics,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider().install_default().unwrap();
    // Initalize environment variables
    let env: service_notes::Env = service_notes::init_envs()?;

    // Initialize tracing + OpenTelemetry
    let meter_provider = service_notes::init_metrics("service-notes");
    let tracer_provider = service_notes::init_tracer("service-notes");
    let tracer = tracer_provider.tracer("service-notes");
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&env.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    // Connect to database
    let pool = service_notes::connect_to_db(&env).context("Failed to connect to database")?;
    tracing::info!("Connected to database");

    // Run migrations
    migrations::run_migrations(&pool)
        .await
        .context("Failed to run migrations")?;
    tracing::info!("Migrations complete");

    // Run gRPC server
    let addr = format!("[::]:{}", env.port).parse()?;
    tracing::info!("gRPC server started on port: {:?}", env.port);
    let server = MyService { pool, env, metrics: service_notes::Metrics::new("service-notes") };
    let svc = NotesServiceServer::new(server);
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .context("Failed to start gRPC server")?;
    tracer_provider.shutdown()?;
    meter_provider.shutdown()?;
    Ok(())
}
