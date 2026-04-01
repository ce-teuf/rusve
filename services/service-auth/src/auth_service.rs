use crate::{
    auth_oauth::{OAuth, OAuthConfig},
    proto::users_service_client::UsersServiceClient,
    AppState,
};
use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Redirect,
};
use oauth2::{CsrfToken, PkceCodeChallenge, Scope, TokenResponse};
use opentelemetry::{global, KeyValue};
use std::{collections::HashMap, sync::Arc};
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Prefix added to the OAuth state parameter when the login came from the mobile app.
/// The callback uses this to redirect to the deep link instead of the web URL.
const MOBILE_STATE_PREFIX: &str = "mobile:";

#[tracing::instrument(skip(state, headers, query), fields(provider = %provider))]
pub async fn oauth_login(
    Path(provider): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Redirect, Redirect> {
    let start = std::time::Instant::now();
    let is_mobile = query.get("mobile").map(|v| v == "true").unwrap_or(false);

    let parent_cx = global::get_text_map_propagator(|prop| {
        prop.extract(&service_auth::HeaderExtractor(&headers))
    });
    tracing::Span::current().set_parent(parent_cx);

    let conn = state.pool.get().await.map_err(|err| {
        tracing::error!("Failed to get DB connection: {:?}", err);
        Redirect::to(&format!("{}/auth?error=1", state.env.client_url))
    })?;

    let oauth_config =
        OAuthConfig::get_config_by_provider(&provider, state.env.clone()).map_err(|err| {
            tracing::error!("Failed to get OAuth provider: {:?}", err);
            Redirect::to(&format!("{}/auth?error=1", state.env.client_url))
        })?;
    let client = oauth_config.build_oauth_client();

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let mut client = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge);
    for scope in oauth_config.scopes {
        client = client.add_scope(Scope::new(scope));
    }
    let (auth_url, csrf_token) = client.add_extra_param("access_type", "offline").url();

    // When the request comes from mobile, prefix the CSRF token with "mobile:" so
    // the callback knows to redirect to the deep link instead of the web URL.
    // The prefix is stored in the DB as part of the csrf_token key.
    let csrf_key = if is_mobile {
        format!("{}{}", MOBILE_STATE_PREFIX, csrf_token.secret())
    } else {
        csrf_token.secret().to_owned()
    };

    // Save the CSRF token and PKCE verifier so we can verify them later.
    crate::auth_db::create_verifiers(&conn, &csrf_key, pkce_verifier.secret())
        .await
        .map_err(|err| {
            tracing::error!("Failed to save verifier: {:?}", err);
            Redirect::to(&format!("{}/auth?error=1", state.env.client_url))
        })?;

    // Replace the state in the URL with our prefixed key so the callback receives it.
    let auth_url = if is_mobile {
        auth_url
            .as_str()
            .replace(csrf_token.secret(), &csrf_key)
    } else {
        auth_url.to_string()
    };

    state.metrics.requests_total.add(1, &[
        KeyValue::new("handler", "oauth_login"),
        KeyValue::new("status", "ok"),
    ]);
    state.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
        KeyValue::new("handler", "oauth_login"),
    ]);
    Ok(Redirect::to(&auth_url))
}

#[tracing::instrument(skip(state, headers, query), fields(provider = %provider))]
pub async fn oauth_callback(
    Path(provider): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Redirect, Redirect> {
    let start = std::time::Instant::now();
    let parent_cx = global::get_text_map_propagator(|prop| {
        prop.extract(&service_auth::HeaderExtractor(&headers))
    });
    tracing::Span::current().set_parent(parent_cx);

    let conn = state.pool.get().await.map_err(|err| {
        tracing::error!("Failed to get DB connection: {:?}", err);
        Redirect::to(&format!("{}/auth?error=2", state.env.client_url))
    })?;

    let code = query.get("code").ok_or_else(|| {
        tracing::error!("Missing code");
        Redirect::to(&format!("{}/auth?error=2", state.env.client_url))
    })?;
    let csrf = query.get("state").ok_or_else(|| {
        tracing::error!("Missing CSRF token");
        Redirect::to(&format!("{}/auth?error=2", state.env.client_url))
    })?;

    // Detect if this callback is for a mobile login (state was prefixed in oauth_login).
    let is_mobile = csrf.starts_with(MOBILE_STATE_PREFIX);

    let verifiers = match crate::auth_db::select_verifiers_by_csrf(&conn, csrf).await {
        Ok(Some(verifiers)) => verifiers,
        Ok(None) => {
            return Err(Redirect::to(&format!(
                "{}/auth?error=2",
                state.env.client_url
            )))
        }
        Err(err) => {
            tracing::error!("Failed to select verifiers: {:?}", err);
            return Err(Redirect::to(&format!(
                "{}/auth?error=2",
                state.env.client_url
            )));
        }
    };

    // Delete old verifiers asynchronously. If this fails, it's not a big deal.
    tokio::spawn(async move {
        if let Err(err) = crate::auth_db::delete_old_verifiers(&conn).await {
            tracing::error!("Failed to delete old verifiers: {:?}", err);
        }
    });

    // Check if the CSRF token is valid.
    if verifiers.created + time::Duration::minutes(10) < time::OffsetDateTime::now_utc() {
        return Err(Redirect::to(&format!(
            "{}/auth?error=2",
            state.env.client_url
        )));
    }

    // Exchange the code with a token.
    let oauth_config =
        OAuthConfig::get_config_by_provider(&provider, state.env.clone()).map_err(|err| {
            tracing::error!("Failed to get OAuth provider: {:?}", err);
            Redirect::to(&format!("{}/auth?error=2", state.env.client_url))
        })?;
    let client = oauth_config.build_oauth_client();
    let token = client
        .exchange_code(oauth2::AuthorizationCode::new(code.to_string()))
        // Set the PKCE code verifier.
        .set_pkce_verifier(oauth2::PkceCodeVerifier::new(verifiers.pkce_verifier))
        .request_async(&reqwest::Client::new())
        .await
        .map_err(|err| {
            tracing::error!("Failed to exchange code with token: {:?}", err);
            Redirect::to(&format!("{}/auth?error=2", state.env.client_url))
        })?;

    // Get the user's profile.
    let user_profile = oauth_config
        .get_user_info(token.access_token().secret())
        .await
        .map_err(|err| {
            tracing::error!("Failed to get user profile: {:?}", err);
            Redirect::to(&format!("{}/auth?error=2", state.env.client_url))
        })?;

    /*
     * This is where you implement you own logic to create or update a user in your database.
     * Here we are calling the users service to create a new user.
     * It returns an id token that we can use to authenticate the user.
     */
    let jwt_token = oauth_config.generate_jwt(user_profile).map_err(|err| {
        tracing::error!("Failed to generate JWT: {:?}", err);
        Redirect::to(&format!("{}/auth?error=2", state.env.client_url))
    })?;
    let client = UsersServiceClient::connect(state.env.users_url.to_owned())
        .await
        .map_err(|err| {
            tracing::error!("Failed to connect to users service: {:?}", err);
            Redirect::to(&format!("{}/auth?error=2", state.env.client_url))
        });
    let mut request = tonic::Request::new(crate::proto::Empty {});
    let metadata = request.metadata_mut();
    metadata.insert("x-authorization", jwt_token);
    // Inject current trace context into the gRPC call to service-users
    let cx = tracing::Span::current().context();
    global::get_text_map_propagator(|prop| {
        prop.inject_context(&cx, &mut service_auth::MetadataInjector(metadata));
    });
    let token = client?
        .create_user(request)
        .await
        .map_err(|err| {
            tracing::error!("Failed to create user: {:?}", err);
            Redirect::to(&format!("{}/auth?error=2", state.env.client_url))
        })?
        .into_inner();

    tracing::info!("User authenticated");
    state.metrics.requests_total.add(1, &[
        KeyValue::new("handler", "oauth_callback"),
        KeyValue::new("status", "ok"),
    ]);
    state.metrics.request_duration_ms.record(start.elapsed().as_millis() as f64, &[
        KeyValue::new("handler", "oauth_callback"),
    ]);

    // Mobile: redirect to the Capacitor deep link so the app can store the token.
    // Web: redirect to the web client with the token as a query param (existing behaviour).
    let redirect_url = if is_mobile {
        format!("com.rusve.app://callback?token={}", token.id)
    } else {
        format!("{}/?token={}", state.env.client_url, token.id)
    };
    Ok(Redirect::to(&redirect_url))
}
