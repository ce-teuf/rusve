use crate::proto::utils_service_server::UtilsService;
use crate::proto::{Count, Email, Empty, File, Id, Page};
use crate::MyService;
use anyhow::Result;
use opentelemetry::{global, KeyValue};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[tonic::async_trait]
impl UtilsService for MyService {
    type GetEmailsByTargetIdStream = ReceiverStream<Result<Email, Status>>;
    type GetFilesByTargetIdStream = ReceiverStream<Result<File, Status>>;
    type GetFileByIdStream = ReceiverStream<Result<File, Status>>;
    type UploadFileStream = ReceiverStream<Result<File, Status>>;

    #[tracing::instrument(skip(self, request), fields(rpc = "count_emails_by_target_id"))]
    async fn count_emails_by_target_id(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Count>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_utils::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::email_service::count_emails_by_target_id(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "count_emails_by_target_id"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "count_emails_by_target_id"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "get_emails_by_target_id"))]
    async fn get_emails_by_target_id(
        &self,
        request: Request<Page>,
    ) -> Result<Response<Self::GetEmailsByTargetIdStream>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_utils::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::email_service::get_emails_by_target_id(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "get_emails_by_target_id"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "get_emails_by_target_id"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "send_email"))]
    async fn send_email(&self, request: Request<Email>) -> Result<Response<Email>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_utils::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::email_service::send_email(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "send_email"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "send_email"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "count_files_by_target_id"))]
    async fn count_files_by_target_id(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<Count>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_utils::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::file_service::count_files_by_target_id(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "count_files_by_target_id"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "count_files_by_target_id"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "get_files_by_target_id"))]
    async fn get_files_by_target_id(
        &self,
        request: Request<Page>,
    ) -> Result<Response<Self::GetFilesByTargetIdStream>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_utils::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::file_service::get_files_by_target_id(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "get_files_by_target_id"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "get_files_by_target_id"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "get_file_by_id"))]
    async fn get_file_by_id(
        &self,
        request: Request<Id>,
    ) -> Result<Response<Self::GetFileByIdStream>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_utils::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::file_service::get_file_by_id(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "get_file_by_id"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "get_file_by_id"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "upload_file"))]
    async fn upload_file(
        &self,
        request: Request<tonic::Streaming<File>>,
    ) -> Result<Response<Self::UploadFileStream>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_utils::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::file_service::upload_file(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "upload_file"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "upload_file"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "delete_file_by_id"))]
    async fn delete_file_by_id(&self, request: Request<Id>) -> Result<Response<Empty>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_utils::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::file_service::delete_file_by_id(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "delete_file_by_id"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "delete_file_by_id"),
        ]);
        result
    }
}
