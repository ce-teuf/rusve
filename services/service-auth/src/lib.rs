use anyhow::{Context, Result};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use opentelemetry::{
    metrics::{Counter, Histogram},
    propagation::{Extractor, Injector},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics::SdkMeterProvider,
    propagation::TraceContextPropagator,
    runtime,
    trace::TracerProvider,
    Resource,
};
use rustls::RootCertStore;
use rustls_native_certs::load_native_certs;
use std::str::FromStr;
use tokio_postgres_rustls::MakeRustlsConnect;

/// Extrait le traceparent depuis les headers HTTP (Axum)
pub struct HeaderExtractor<'a>(pub &'a axum::http::HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }
    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

/// Injecte le traceparent dans les métadonnées gRPC sortantes
pub struct MetadataInjector<'a>(pub &'a mut tonic::metadata::MetadataMap);

impl<'a> Injector for MetadataInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(k), Ok(v)) = (
            key.parse::<tonic::metadata::MetadataKey<tonic::metadata::Ascii>>(),
            value.parse::<tonic::metadata::MetadataValue<tonic::metadata::Ascii>>(),
        ) {
            self.0.insert(k, v);
        }
    }
}

#[derive(Clone)]
pub struct Env {
    pub port: String,
    pub rust_log: String,
    pub database_url: String,
    pub auth_url: String,
    pub client_url: String,
    pub users_url: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub jwt_secret: String,
}

pub fn init_envs() -> Result<Env> {
    Ok(Env {
        port: std::env::var("PORT").context("PORT is not set")?,
        rust_log: std::env::var("RUST_LOG").context("RUST_LOG is not set")?,
        database_url: std::env::var("DATABASE_URL").context("DATABASE_URL is not set")?,
        auth_url: std::env::var("AUTH_URL").context("AUTH_URL is not set")?,
        client_url: std::env::var("CLIENT_URL").context("CLIENT_URL is not set")?,
        users_url: std::env::var("USERS_URL").context("USERS_URL is not set")?,
        google_client_id: std::env::var("GOOGLE_CLIENT_ID")
            .context("GOOGLE_CLIENT_ID is not set")?,
        google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET")
            .context("GOOGLE_CLIENT_SECRET is not set")?,
        github_client_id: std::env::var("GITHUB_CLIENT_ID")
            .context("GITHUB_CLIENT_ID is not set")?,
        github_client_secret: std::env::var("GITHUB_CLIENT_SECRET")
            .context("GITHUB_CLIENT_SECRET is not set")?,
        jwt_secret: std::env::var("JWT_SECRET").context("JWT_SECRET is not set")?,
    })
}

pub struct Metrics {
    pub requests_total: Counter<u64>,
    pub request_duration_ms: Histogram<f64>,
}

impl Metrics {
    pub fn new(service_name: &'static str) -> Self {
        let meter = opentelemetry::global::meter(service_name);
        Self {
            requests_total: meter
                .u64_counter("http_requests_total")
                .with_description("Total number of HTTP requests")
                .build(),
            request_duration_ms: meter
                .f64_histogram("http_request_duration_ms")
                .with_description("HTTP request duration in milliseconds")
                .build(),
        }
    }
}

pub fn init_metrics(service_name: &'static str) -> SdkMeterProvider {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("Failed to build OTLP metric exporter");
    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter, runtime::Tokio)
        .with_interval(std::time::Duration::from_secs(15))
        .build();
    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(Resource::new(vec![KeyValue::new("service.name", service_name)]))
        .build();
    opentelemetry::global::set_meter_provider(provider.clone());
    provider
}

pub fn init_tracer(service_name: &'static str) -> TracerProvider {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("Failed to build OTLP span exporter");
    TracerProvider::builder()
        .with_resource(Resource::new(vec![KeyValue::new("service.name", service_name)]))
        .with_batch_exporter(exporter, runtime::Tokio)
        .build()
}

pub fn connect_to_db(env: &Env) -> Result<deadpool_postgres::Pool> {
    let tokio_config = tokio_postgres::Config::from_str(env.database_url.as_str())?;
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let result = load_native_certs();
    let mut store = RootCertStore::empty();
    for cert in result.certs {
        store.add(cert)?;
    }
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(store)
        .with_no_client_auth();
    let tls = MakeRustlsConnect::new(config);
    let mgr = Manager::from_config(tokio_config, tls, mgr_config);
    let pool = Pool::builder(mgr).build()?;
    Ok(pool)
}
