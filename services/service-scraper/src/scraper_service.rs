use crate::{
    proto::{
        scraper_service_server::ScraperService, Count, Empty, Id, Item, ItemFilter, Job,
        JobResponse, Page, Source,
    },
    MyService, SchedulerState,
};
use anyhow::Result;
use opentelemetry::KeyValue;
use tokio::sync::mpsc;
use tokio_cron_scheduler::Job as CronJob;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

fn internal(msg: &str, e: impl std::fmt::Debug) -> Status {
    tracing::error!("{}: {:?}", msg, e);
    Status::internal(msg)
}

async fn register_source_cron(
    state: &mut SchedulerState,
    source: &Source,
    pool: deadpool_postgres::Pool,
    data_pool: deadpool_postgres::Pool,
) {
    if source.integration_mode != "AUTO" || source.auto_schedule.is_empty() || !source.active {
        return;
    }
    let source_id = source.id.clone();
    let pool_c = pool.clone();
    let data_pool_c = data_pool.clone();

    let job = CronJob::new_async(source.auto_schedule.as_str(), move |_uuid, _lock| {
        let source_id = source_id.clone();
        let pool = pool_c.clone();
        let data_pool = data_pool_c.clone();
        Box::pin(async move {
            if let Err(e) = crate::scraper_db::auto_push_source(&source_id, &pool, &data_pool).await {
                tracing::error!("Auto push failed for source {}: {:?}", source_id, e);
            }
        })
    });

    match job {
        Ok(j) => {
            match state.scheduler.add(j).await {
                Ok(uuid) => {
                    state.jobs.insert(source.id.clone(), uuid);
                    tracing::info!("Registered cron job for source {}: {}", source.id, source.auto_schedule);
                }
                Err(e) => tracing::error!("Failed to add cron job: {:?}", e),
            }
        }
        Err(e) => tracing::error!("Invalid cron expression '{}': {:?}", source.auto_schedule, e),
    }
}

async fn remove_source_cron(state: &mut SchedulerState, source_id: &str) {
    if let Some(uuid) = state.jobs.remove(source_id) {
        if let Err(e) = state.scheduler.remove(&uuid).await {
            tracing::error!("Failed to remove cron job: {:?}", e);
        }
    }
}

#[tonic::async_trait]
impl ScraperService for MyService {
    type ListSourcesStream = ReceiverStream<Result<Source, Status>>;
    type ListJobsStream = ReceiverStream<Result<JobResponse, Status>>;
    type ListItemsStream = ReceiverStream<Result<Item, Status>>;

    // ── Sources ──────────────────────────────────────────────────

    #[tracing::instrument(skip(self, request), fields(rpc = "list_sources"))]
    async fn list_sources(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Self::ListSourcesStream>, Status> {
        let start = std::time::Instant::now();
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;

        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;
        let sources = crate::scraper_db::list_sources(&conn)
            .await
            .map_err(|e| internal("Failed to list sources", e))?;

        let (tx, rx) = mpsc::channel(64);
        tokio::spawn(async move {
            for s in sources {
                if tx.send(Ok(s)).await.is_err() {
                    break;
                }
            }
        });

        self.metrics.requests_total.add(1, &[KeyValue::new("method", "list_sources")]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[KeyValue::new("method", "list_sources")]);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "get_source"))]
    async fn get_source(&self, request: Request<Id>) -> Result<Response<Source>, Status> {
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;
        let id = request.into_inner();
        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;
        let source = crate::scraper_db::get_source(&conn, &id.id)
            .await
            .map_err(|e| internal("Failed to get source", e))?;
        Ok(Response::new(source))
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "create_source"))]
    async fn create_source(&self, request: Request<Source>) -> Result<Response<Source>, Status> {
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;
        let source_req = request.into_inner();
        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;
        let source = crate::scraper_db::insert_source(&conn, &source_req)
            .await
            .map_err(|e| internal("Failed to create source", e))?;

        let mut state = self.scheduler.lock().await;
        register_source_cron(&mut state, &source, self.pool.clone(), self.data_pool.clone()).await;

        Ok(Response::new(source))
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "update_source"))]
    async fn update_source(&self, request: Request<Source>) -> Result<Response<Source>, Status> {
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;
        let source_req = request.into_inner();
        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;
        let source = crate::scraper_db::update_source(&conn, &source_req)
            .await
            .map_err(|e| internal("Failed to update source", e))?;

        // Reload cron job
        let mut state = self.scheduler.lock().await;
        remove_source_cron(&mut state, &source.id).await;
        register_source_cron(&mut state, &source, self.pool.clone(), self.data_pool.clone()).await;

        Ok(Response::new(source))
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "delete_source"))]
    async fn delete_source(&self, request: Request<Id>) -> Result<Response<Empty>, Status> {
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;
        let id = request.into_inner();
        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;

        let mut state = self.scheduler.lock().await;
        remove_source_cron(&mut state, &id.id).await;
        drop(state);

        crate::scraper_db::delete_source(&conn, &id.id)
            .await
            .map_err(|e| internal("Failed to delete source", e))?;
        Ok(Response::new(Empty {}))
    }

    // ── Jobs ─────────────────────────────────────────────────────

    #[tracing::instrument(skip(self, request), fields(rpc = "list_jobs"))]
    async fn list_jobs(
        &self,
        request: Request<Page>,
    ) -> Result<Response<Self::ListJobsStream>, Status> {
        let start = std::time::Instant::now();
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;

        let page = request.into_inner();
        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;
        let jobs = crate::scraper_db::list_jobs(&conn, page.offset, page.limit)
            .await
            .map_err(|e| internal("Failed to list jobs", e))?;

        let (tx, rx) = mpsc::channel(64);
        tokio::spawn(async move {
            for j in jobs {
                if tx.send(Ok(j)).await.is_err() {
                    break;
                }
            }
        });

        self.metrics.requests_total.add(1, &[KeyValue::new("method", "list_jobs")]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[KeyValue::new("method", "list_jobs")]);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "get_job_by_id"))]
    async fn get_job_by_id(&self, request: Request<Id>) -> Result<Response<Job>, Status> {
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;
        let id = request.into_inner();
        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;
        let job = crate::scraper_db::get_job(&conn, &id.id)
            .await
            .map_err(|e| internal("Failed to get job", e))?;
        Ok(Response::new(job))
    }

    // ── Items ────────────────────────────────────────────────────

    #[tracing::instrument(skip(self, request), fields(rpc = "list_items"))]
    async fn list_items(
        &self,
        request: Request<ItemFilter>,
    ) -> Result<Response<Self::ListItemsStream>, Status> {
        let start = std::time::Instant::now();
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;

        let filter = request.into_inner();
        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;
        let items = crate::scraper_db::list_items(&conn, &filter.job_id, &filter.status, filter.offset, filter.limit)
            .await
            .map_err(|e| internal("Failed to list items", e))?;

        let (tx, rx) = mpsc::channel(128);
        tokio::spawn(async move {
            for item in items {
                if tx.send(Ok(item)).await.is_err() {
                    break;
                }
            }
        });

        self.metrics.requests_total.add(1, &[KeyValue::new("method", "list_items")]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[KeyValue::new("method", "list_items")]);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "approve_item"))]
    async fn approve_item(&self, request: Request<Id>) -> Result<Response<Item>, Status> {
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;
        let id = request.into_inner();
        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;
        let item = crate::scraper_db::update_item_status(&conn, &id.id, "APPROVED", "[]")
            .await
            .map_err(|e| internal("Failed to approve item", e))?;
        Ok(Response::new(item))
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "reject_item"))]
    async fn reject_item(&self, request: Request<Id>) -> Result<Response<Item>, Status> {
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;
        let id = request.into_inner();
        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;
        let item = crate::scraper_db::update_item_status(&conn, &id.id, "REJECTED", "[]")
            .await
            .map_err(|e| internal("Failed to reject item", e))?;
        Ok(Response::new(item))
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "approve_all_valid"))]
    async fn approve_all_valid(&self, request: Request<Id>) -> Result<Response<Count>, Status> {
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;
        let id = request.into_inner();
        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;

        // Validate any PENDING items first
        let field_rules = crate::scraper_db::get_source_field_rules_by_job(&conn, &id.id)
            .await
            .unwrap_or_else(|_| "[]".to_string());
        let pending = crate::scraper_db::get_pending_items_for_job(&conn, &id.id)
            .await
            .map_err(|e| internal("Failed to get pending items", e))?;

        for item in &pending {
            let raw: serde_json::Value = serde_json::from_str(&item.raw_data).unwrap_or(serde_json::json!({}));
            let (status, errors) = crate::scraper_validation::validate(&raw, &field_rules);
            let errors_json = serde_json::to_string(&errors).unwrap_or_else(|_| "[]".to_string());
            let _ = crate::scraper_db::update_item_status(&conn, &item.id, &status, &errors_json).await;
        }

        let count = crate::scraper_db::approve_all_valid(&conn, &id.id)
            .await
            .map_err(|e| internal("Failed to approve all valid", e))?;
        Ok(Response::new(Count { count }))
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "push_approved"))]
    async fn push_approved(&self, request: Request<Id>) -> Result<Response<Count>, Status> {
        service_scraper::auth(request.metadata(), &self.env.jwt_secret)?;
        let id = request.into_inner();

        let conn = self.pool.get().await.map_err(|e| internal("Failed to get connection", e))?;
        let source_type = crate::scraper_db::get_job_source_type(&conn, &id.id)
            .await
            .map_err(|e| internal("Failed to get job", e))?;
        let items = crate::scraper_db::get_approved_items_for_job(&conn, &id.id)
            .await
            .map_err(|e| internal("Failed to get approved items", e))?;

        if items.is_empty() {
            return Ok(Response::new(Count { count: 0 }));
        }

        let data_conn = self.data_pool.get().await.map_err(|e| internal("Failed to get data connection", e))?;
        let mut pushed_ids = Vec::with_capacity(items.len());

        for item in &items {
            crate::scraper_db::insert_data_item(&data_conn, &source_type, &item.raw_data, &item.id)
                .await
                .map_err(|e| internal("Failed to insert data item", e))?;
            pushed_ids.push(item.id.clone());
        }

        crate::scraper_db::mark_items_pushed(&conn, &pushed_ids, "db_data")
            .await
            .map_err(|e| internal("Failed to mark items pushed", e))?;

        Ok(Response::new(Count { count: pushed_ids.len() as i64 }))
    }
}
