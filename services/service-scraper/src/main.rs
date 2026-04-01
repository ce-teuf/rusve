mod migrations;
mod scraper_db;
mod scraper_service;
mod scraper_validation;
mod proto;

use crate::proto::scraper_service_server::ScraperServiceServer;
use anyhow::{Context, Result};
use opentelemetry::trace::TracerProvider as _;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tokio_cron_scheduler::JobScheduler;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;

pub struct SchedulerState {
    pub scheduler: JobScheduler,
    pub jobs: HashMap<String, uuid::Uuid>,
}

pub struct MyService {
    env: service_scraper::Env,
    pool: deadpool_postgres::Pool,
    data_pool: deadpool_postgres::Pool,
    metrics: service_scraper::Metrics,
    scheduler: Arc<Mutex<SchedulerState>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider().install_default().unwrap();

    let env = service_scraper::init_envs()?;

    let meter_provider = service_scraper::init_metrics("service-scraper");
    let tracer_provider = service_scraper::init_tracer("service-scraper");
    let tracer = tracer_provider.tracer("service-scraper");
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&env.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    let pool = service_scraper::connect_to_db(&env.database_url)
        .context("Failed to connect to db_scraping")?;
    let data_pool = service_scraper::connect_to_db(&env.data_database_url)
        .context("Failed to connect to db_data")?;

    tracing::info!("Connected to databases");

    migrations::run_migrations(&pool)
        .await
        .context("Failed to run scraping migrations")?;
    migrations::run_data_migrations(&data_pool)
        .await
        .context("Failed to run data migrations")?;
    tracing::info!("Migrations complete");

    // Initialize scheduler and load AUTO sources
    let sched = JobScheduler::new().await.context("Failed to create scheduler")?;
    let mut sched_jobs: HashMap<String, uuid::Uuid> = HashMap::new();

    {
        let conn = pool.get().await.context("Failed to get connection")?;
        let auto_sources = scraper_db::list_auto_sources(&conn)
            .await
            .context("Failed to load auto sources")?;

        for source in &auto_sources {
            if source.auto_schedule.is_empty() {
                continue;
            }
            let source_id = source.id.clone();
            let pool_c = pool.clone();
            let data_pool_c = data_pool.clone();

            let job = tokio_cron_scheduler::Job::new_async(
                source.auto_schedule.as_str(),
                move |_uuid, _lock| {
                    let source_id = source_id.clone();
                    let pool = pool_c.clone();
                    let data_pool = data_pool_c.clone();
                    Box::pin(async move {
                        if let Err(e) = scraper_db::auto_push_source(&source_id, &pool, &data_pool).await {
                            tracing::error!("Auto push failed for source {}: {:?}", source_id, e);
                        }
                    })
                },
            );

            match job {
                Ok(j) => {
                    match sched.add(j).await {
                        Ok(uuid) => {
                            sched_jobs.insert(source.id.clone(), uuid);
                            tracing::info!("Registered cron '{}' for source {}", source.auto_schedule, source.id);
                        }
                        Err(e) => tracing::error!("Failed to add cron job: {:?}", e),
                    }
                }
                Err(e) => tracing::error!("Invalid cron '{}': {:?}", source.auto_schedule, e),
            }
        }
    }

    sched.start().await.context("Failed to start scheduler")?;
    tracing::info!("Scheduler started with {} jobs", sched_jobs.len());

    let scheduler_state = Arc::new(Mutex::new(SchedulerState {
        scheduler: sched,
        jobs: sched_jobs,
    }));

    let addr = format!("[::]:{}", env.port).parse()?;
    tracing::info!("gRPC server started on port: {}", env.port);

    let server = MyService {
        pool,
        data_pool,
        env,
        metrics: service_scraper::Metrics::new("service-scraper"),
        scheduler: scheduler_state,
    };

    let svc = ScraperServiceServer::new(server);
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .context("Failed to start gRPC server")?;

    tracer_provider.shutdown()?;
    meter_provider.shutdown()?;
    Ok(())
}
