use crate::proto::users_service_server::UsersService;
use crate::proto::{AuthResponse, Empty, Id, Profile};
use crate::MyService;
use anyhow::Result;
use opentelemetry::{global, KeyValue};
use tonic::{Request, Response, Status};
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[tonic::async_trait]
impl UsersService for MyService {
    #[tracing::instrument(skip(self, request), fields(rpc = "create_user"))]
    async fn create_user(
        &self,
        request: Request<crate::proto::Empty>,
    ) -> Result<Response<Id>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_users::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::user_service::create_user(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "create_user"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "create_user"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "auth"))]
    async fn auth(&self, request: Request<Empty>) -> Result<Response<AuthResponse>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_users::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::user_service::auth(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "auth"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "auth"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "get_profile_by_user_id"))]
    async fn get_profile_by_user_id(
        &self,
        request: Request<crate::proto::Empty>,
    ) -> Result<Response<crate::proto::Profile>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_users::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::profile_service::get_profile_by_user_id(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "get_profile_by_user_id"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "get_profile_by_user_id"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "create_profile"))]
    async fn create_profile(
        &self,
        request: Request<Profile>,
    ) -> Result<Response<Profile>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_users::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::profile_service::create_profile(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "create_profile"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "create_profile"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "create_stripe_checkout"))]
    async fn create_stripe_checkout(
        &self,
        request: Request<crate::proto::Empty>,
    ) -> Result<Response<crate::proto::StripeUrlResponse>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_users::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::stripe_service::create_stripe_checkout(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "create_stripe_checkout"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "create_stripe_checkout"),
        ]);
        result
    }

    #[tracing::instrument(skip(self, request), fields(rpc = "create_stripe_portal"))]
    async fn create_stripe_portal(
        &self,
        request: Request<crate::proto::Empty>,
    ) -> Result<Response<crate::proto::StripeUrlResponse>, Status> {
        let start = std::time::Instant::now();
        let parent_cx = global::get_text_map_propagator(|prop| {
            prop.extract(&service_users::MetadataExtractor(request.metadata()))
        });
        tracing::Span::current().set_parent(parent_cx);
        let result = crate::stripe_service::create_stripe_portal(&self.env, &self.pool, request).await;
        let status = if result.is_ok() { "ok" } else { "error" };
        self.metrics.requests_total.add(1, &[
            KeyValue::new("method", "create_stripe_portal"),
            KeyValue::new("status", status),
        ]);
        self.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
            KeyValue::new("method", "create_stripe_portal"),
        ]);
        result
    }
}
