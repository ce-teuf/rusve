use crate::{
    auth_oauth::OAuthUser,
    proto::users_service_client::UsersServiceClient,
    proto::utils_service_client::UtilsServiceClient,
    proto::Email,
    AppState,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, http::StatusCode, Json};
use deadpool_postgres::Object;
use opentelemetry::KeyValue;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tonic::metadata::{Ascii, MetadataValue};
use uuid::Uuid;

const VERIFICATION_CODE_LENGTH: usize = 6;
const CODE_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ123456789";

#[derive(Deserialize)]
pub struct RegisterBody {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct VerifyBody {
    pub email: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ErrorResponse>)>;

fn api_err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (status, Json(ErrorResponse { error: msg.to_owned() }))
}

pub struct LocalCredential {
    pub password_hash: String,
    pub verified: bool,
}

async fn get_credential(conn: &Object, email: &str) -> anyhow::Result<Option<LocalCredential>> {
    let row = conn
        .query_opt(
            "select password_hash, verified from local_credentials where email = $1",
            &[&email],
        )
        .await?;
    Ok(row.map(|r| LocalCredential {
        password_hash: r.get("password_hash"),
        verified: r.get("verified"),
    }))
}

async fn insert_credential(conn: &Object, email: &str, password_hash: &str) -> anyhow::Result<()> {
    let id = Uuid::now_v7();
    conn.execute(
        "insert into local_credentials (id, email, password_hash, verified) values ($1, $2, $3, false)",
        &[&id, &email, &password_hash],
    )
    .await?;
    Ok(())
}

async fn update_verified(conn: &Object, email: &str) -> anyhow::Result<()> {
    conn.execute(
        "update local_credentials set verified = true where email = $1",
        &[&email],
    )
    .await?;
    Ok(())
}

fn generate_verification_code() -> String {
    let mut rng = rand::thread_rng();
    (0..VERIFICATION_CODE_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CODE_CHARSET.len());
            CODE_CHARSET[idx] as char
        })
        .collect()
}

async fn insert_verification_code(conn: &Object, email: &str, code: &str) -> anyhow::Result<()> {
    let id = Uuid::now_v7();
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::minutes(15);
    conn.execute(
        "insert into verification_codes (id, email, code, expires_at) values ($1, $2, $3, $4)",
        &[&id, &email, &code, &expires_at],
    )
    .await?;
    Ok(())
}

async fn verify_code(conn: &Object, email: &str, code: &str) -> anyhow::Result<bool> {
    let row = conn
        .query_opt(
            "select code from verification_codes where email = $1 and expires_at > now() order by created desc limit 1",
            &[&email],
        )
        .await?;
    
    match row {
        Some(r) => Ok(r.get::<_, String>("code") == code),
        None => Ok(false),
    }
}

async fn delete_verification_codes(conn: &Object, email: &str) -> anyhow::Result<()> {
    conn.execute(
        "delete from verification_codes where email = $1",
        &[&email],
    )
    .await?;
    Ok(())
}

async fn send_verification_email(
    utils_url: &str,
    email: &str,
    code: &str,
    jwt: MetadataValue<Ascii>,
) -> anyhow::Result<()> {
    let mut client = UtilsServiceClient::connect(utils_url.to_owned()).await?;
    
    let email_body = format!(
        r#"<html>
<body>
<p>Your verification code is: <strong style="font-size: 24px; letter-spacing: 4px;">{}</strong></p>
<p>This code expires in 15 minutes.</p>
<p>If you did not create an account, please ignore this email.</p>
</body>
</html>"#,
        code
    );
    
    let email_msg = Email {
        id: String::new(),
        created: String::new(),
        updated: String::new(),
        deleted: String::new(),
        target_id: String::new(),
        email_to: email.to_string(),
        email_from: "noreply@upsend.app".to_string(),
        email_from_name: "UpSend".to_string(),
        email_subject: "Verify your account".to_string(),
        email_body,
    };
    
    let mut req = tonic::Request::new(email_msg);
    req.metadata_mut().insert("x-authorization", jwt);
    client.send_email(req).await?;
    Ok(())
}

fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 12 {
        return Err("Password must be at least 12 characters");
    }
    let digit_count = password.chars().filter(|c| c.is_ascii_digit()).count();
    if digit_count < 2 {
        return Err("Password must contain at least 2 digits");
    }
    let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
    if !password.chars().any(|c| special_chars.contains(c)) {
        return Err("Password must contain at least 1 special character");
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err("Password must contain at least 1 uppercase letter");
    }
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err("Password must contain at least 1 lowercase letter");
    }
    Ok(())
}

fn build_jwt(email: &str, jwt_secret: &str) -> anyhow::Result<MetadataValue<Ascii>> {
    let claims = OAuthUser {
        sub: format!("local:{}", email),
        email: email.to_owned(),
        avatar: String::new(),
        exp: time::OffsetDateTime::now_utc().unix_timestamp() + 60 * 5,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    Ok(format!("bearer {}", token).parse()?)
}

async fn issue_session_token(users_url: &str, jwt: MetadataValue<Ascii>) -> anyhow::Result<String> {
    let mut client = UsersServiceClient::connect(users_url.to_owned()).await?;
    let mut req = tonic::Request::new(crate::proto::Empty {});
    req.metadata_mut().insert("x-authorization", jwt);
    Ok(client.create_user(req).await?.into_inner().id)
}

#[derive(Deserialize)]
pub struct ResendBody {
    pub email: String,
}

#[tracing::instrument(skip(state, body))]
pub async fn resend_verification(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ResendBody>,
) -> ApiResult<MessageResponse> {
    let email = body.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err(api_err(StatusCode::BAD_REQUEST, "Invalid email address"));
    }

    let conn = state.pool.get().await.map_err(|e| {
        tracing::error!("DB pool error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let credential = get_credential(&conn, &email).await.map_err(|e| {
        tracing::error!("DB query error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    match credential {
        None => return Err(api_err(StatusCode::NOT_FOUND, "Account not found")),
        Some(c) if c.verified => return Err(api_err(StatusCode::BAD_REQUEST, "Account is already verified")),
        _ => {}
    }

    delete_verification_codes(&conn, &email).await.map_err(|e| {
        tracing::error!("Failed to delete old codes: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let code = generate_verification_code();
    insert_verification_code(&conn, &email, &code).await.map_err(|e| {
        tracing::error!("Failed to insert verification code: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let jwt = build_jwt(&email, &state.env.jwt_secret).map_err(|e| {
        tracing::error!("JWT error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    send_verification_email(&state.env.utils_url, &email, &code, jwt).await.map_err(|e| {
        tracing::error!("Failed to send verification email: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    Ok(Json(MessageResponse { message: "Verification code resent".to_string() }))
}

#[tracing::instrument(skip(state, body))]
pub async fn local_register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterBody>,
) -> ApiResult<MessageResponse> {
    let start = std::time::Instant::now();

    let email = body.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') || !email.contains('.') {
        return Err(api_err(StatusCode::BAD_REQUEST, "Invalid email address"));
    }

    if let Err(e) = validate_password(&body.password) {
        return Err(api_err(StatusCode::BAD_REQUEST, e));
    }

    let conn = state.pool.get().await.map_err(|e| {
        tracing::error!("DB pool error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let existing = get_credential(&conn, &email).await.map_err(|e| {
        tracing::error!("DB query error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    if let Some(ref cred) = existing {
        if cred.verified {
            return Err(api_err(StatusCode::CONFLICT, "An account with this email already exists"));
        }
        // Unverified: check if a code was sent in the last 60s to prevent duplicate emails
        let recent = conn
            .query_opt(
                "select id from verification_codes where email = $1 and created > now() - interval '60 seconds'",
                &[&email],
            )
            .await
            .map_err(|e| {
                tracing::error!("DB query error: {:?}", e);
                api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
            })?;
        if recent.is_some() {
            // Email was sent very recently — silently succeed to avoid double send
            return Ok(Json(MessageResponse { message: "Verification code sent to your email".to_string() }));
        }
        // No recent code — clean up and allow re-registration with new password
        conn.execute("delete from verification_codes where email = $1", &[&email])
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete old codes: {:?}", e);
                api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
            })?;
        conn.execute("delete from local_credentials where email = $1", &[&email])
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete old credential: {:?}", e);
                api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
            })?;
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!("Argon2 error: {:?}", e);
            api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
        })?
        .to_string();

    insert_credential(&conn, &email, &password_hash).await.map_err(|e| {
        tracing::error!("DB insert error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let code = generate_verification_code();
    insert_verification_code(&conn, &email, &code).await.map_err(|e| {
        tracing::error!("Failed to insert verification code: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let jwt = build_jwt(&email, &state.env.jwt_secret).map_err(|e| {
        tracing::error!("JWT error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    send_verification_email(&state.env.utils_url, &email, &code, jwt.clone()).await.map_err(|e| {
        tracing::error!("Failed to send verification email: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    tracing::info!("Verification code sent to {}", email);
    state.metrics.requests_total.add(1, &[
        KeyValue::new("handler", "local_register"),
        KeyValue::new("status", "ok"),
    ]);
    state.metrics.request_duration_ms.record(
        start.elapsed().as_millis() as f64,
        &[KeyValue::new("handler", "local_register")],
    );

    Ok(Json(MessageResponse { message: "Verification code sent to your email".to_string() }))
}

#[tracing::instrument(skip(state, body))]
pub async fn verify_account(
    State(state): State<Arc<AppState>>,
    Json(body): Json<VerifyBody>,
) -> ApiResult<TokenResponse> {
    let start = std::time::Instant::now();

    let email = body.email.trim().to_lowercase();
    let code = body.code.trim().to_uppercase();

    if email.is_empty() || code.is_empty() {
        return Err(api_err(StatusCode::BAD_REQUEST, "Email and code are required"));
    }

    let conn = state.pool.get().await.map_err(|e| {
        tracing::error!("DB pool error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let credential = get_credential(&conn, &email).await.map_err(|e| {
        tracing::error!("DB query error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    if credential.is_none() {
        return Err(api_err(StatusCode::NOT_FOUND, "Account not found"));
    }

    if credential.as_ref().unwrap().verified {
        return Err(api_err(StatusCode::BAD_REQUEST, "Account is already verified"));
    }

    if !verify_code(&conn, &email, &code).await.map_err(|e| {
        tracing::error!("Code verification error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })? {
        return Err(api_err(StatusCode::BAD_REQUEST, "Invalid or expired verification code"));
    }

    update_verified(&conn, &email).await.map_err(|e| {
        tracing::error!("Failed to update verified status: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    delete_verification_codes(&conn, &email).await.map_err(|e| {
        tracing::error!("Failed to delete verification codes: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let jwt = build_jwt(&email, &state.env.jwt_secret).map_err(|e| {
        tracing::error!("JWT error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let token = issue_session_token(&state.env.users_url, jwt).await.map_err(|e| {
        tracing::error!("Token issuance error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    tracing::info!("Local user verified");
    state.metrics.requests_total.add(1, &[
        KeyValue::new("handler", "verify_account"),
        KeyValue::new("status", "ok"),
    ]);
    state.metrics.request_duration_ms.record(
        start.elapsed().as_millis() as f64,
        &[KeyValue::new("handler", "verify_account")],
    );

    Ok(Json(TokenResponse { token }))
}

#[tracing::instrument(skip(state, body))]
pub async fn local_login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginBody>,
) -> ApiResult<TokenResponse> {
    let start = std::time::Instant::now();

    let email = body.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(api_err(StatusCode::BAD_REQUEST, "Email is required"));
    }

    let conn = state.pool.get().await.map_err(|e| {
        tracing::error!("DB pool error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let credential = get_credential(&conn, &email).await.map_err(|e| {
        tracing::error!("DB query error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let hash_to_verify = credential
        .as_ref()
        .map(|c| c.password_hash.as_str())
        .unwrap_or(&state.dummy_hash);

    let parsed = PasswordHash::new(hash_to_verify).map_err(|e| {
        tracing::error!("Hash parse error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let verified = Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed)
        .is_ok();

    if credential.is_none() || !verified {
        return Err(api_err(StatusCode::UNAUTHORIZED, "Invalid email or password"));
    }

    if !credential.unwrap().verified {
        return Err(api_err(StatusCode::FORBIDDEN, "Please verify your email first"));
    }

    let jwt = build_jwt(&email, &state.env.jwt_secret).map_err(|e| {
        tracing::error!("JWT error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    let token = issue_session_token(&state.env.users_url, jwt).await.map_err(|e| {
        tracing::error!("Token issuance error: {:?}", e);
        api_err(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
    })?;

    tracing::info!("Local user authenticated");
    state.metrics.requests_total.add(1, &[
        KeyValue::new("handler", "local_login"),
        KeyValue::new("status", "ok"),
    ]);
    state.metrics.request_duration_ms.record(
        start.elapsed().as_millis() as f64,
        &[KeyValue::new("handler", "local_login")],
    );

    Ok(Json(TokenResponse { token }))
}