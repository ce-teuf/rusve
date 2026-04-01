use anyhow::{Context, Result};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use opentelemetry::{
    metrics::{Counter, Histogram},
    propagation::Extractor,
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
use std::str::FromStr;
mod proto;

pub struct MetadataExtractor<'a>(pub &'a tonic::metadata::MetadataMap);

impl<'a> Extractor for MetadataExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }
    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .filter_map(|k| {
                if let tonic::metadata::KeyRef::Ascii(k) = k {
                    Some(k.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct Env {
    pub port: String,
    pub rust_log: String,
    pub database_url: String,
    pub sendgrid_api_key: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub s3_bucket_name: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_endpoint: String,
    pub jwt_secret: String,
}

pub fn init_envs() -> Result<Env> {
    Ok(Env {
        port: std::env::var("PORT").context("PORT is not set")?,
        rust_log: std::env::var("RUST_LOG").context("RUST_LOG is not set")?,
        database_url: std::env::var("DATABASE_URL").context("DATABASE_URL is not set")?,
        sendgrid_api_key: std::env::var("SENDGRID_API_KEY")
            .context("SENDGRID_API_KEY is not set")?,
        smtp_host: std::env::var("SMTP_HOST").unwrap_or_default(),
        smtp_port: std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .unwrap_or(587),
        smtp_username: std::env::var("SMTP_USERNAME").unwrap_or_default(),
        smtp_password: std::env::var("SMTP_PASSWORD").unwrap_or_default(),
        smtp_from_email: std::env::var("SMTP_FROM_EMAIL").unwrap_or_default(),
        smtp_from_name: std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "UpSend".to_string()),
        s3_bucket_name: std::env::var("S3_BUCKET_NAME").context("S3_BUCKET_NAME is not set")?,
        s3_access_key: std::env::var("S3_ACCESS_KEY").context("S3_ACCESS_KEY is not set")?,
        s3_secret_key: std::env::var("S3_SECRET_KEY").context("S3_SECRET_KEY is not set")?,
        s3_endpoint: std::env::var("S3_ENDPOINT").context("S3_ENDPOINT is not set")?,
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
                .u64_counter("grpc_requests_total")
                .with_description("Total number of gRPC requests")
                .build(),
            request_duration_ms: meter
                .f64_histogram("grpc_request_duration_ms")
                .with_description("gRPC request duration in milliseconds")
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
    let tokio_config = tokio_postgres::Config::from_str(&env.database_url)?;
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let result = rustls_native_certs::load_native_certs();
    let mut store = rustls::RootCertStore::empty();
    for cert in result.certs {
        store.add(cert)?;
    }
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(store)
        .with_no_client_auth();
    let tls = tokio_postgres_rustls::MakeRustlsConnect::new(config);
    let mgr = Manager::from_config(tokio_config, tls, mgr_config);
    let pool = Pool::builder(mgr).build()?;
    Ok(pool)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OAuthClaims {
    pub sub: String,
    pub email: String,
    pub avatar: String,
}
pub fn auth(
    metadata: &tonic::metadata::MetadataMap,
    jwt_secret: &str,
) -> Result<OAuthClaims, tonic::Status> {
    let token = match metadata.get("x-authorization") {
        Some(token) => token,
        None => {
            tracing::error!("Missing authorization token");
            return Err(tonic::Status::unauthenticated(
                "Missing authorization token",
            ));
        }
    };
    let token = token
        .to_str()
        .map_err(|e| {
            tracing::error!("Failed to parse authorization token: {:?}", e);
            tonic::Status::unauthenticated("Invalid authorization token")
        })?
        .strip_prefix("bearer ")
        .ok_or_else(|| {
            tracing::error!("Failed to parse authorization token");
            tonic::Status::unauthenticated("Invalid authorization token")
        })?;

    let token_message = jsonwebtoken::decode::<OAuthClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_ref()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map_err(|e| {
        tracing::error!("Failed to decode authorization token: {:?}", e);
        tonic::Status::unauthenticated("Invalid authorization token")
    })?;

    Ok(token_message.claims)
}
