use anyhow::Result;
use sendgrid::{Destination, Mail, SGClient};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::proto::{Count, Email, Empty, Page};

fn is_smtp_configured(env: &service_utils::Env) -> bool {
    !env.smtp_password.is_empty() && !env.smtp_host.is_empty()
}

use lettre::message::{Mailbox, MessageBuilder};

async fn send_via_smtp(env: &service_utils::Env, email: &Email) -> Result<(), String> {
    use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
    use lettre::transport::smtp::authentication::Credentials;

    let credentials = Credentials::new(env.smtp_username.clone(), env.smtp_password.clone());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&env.smtp_host)
        .map_err(|e| format!("Failed to create SMTP relay: {}", e))?
        .port(env.smtp_port)
        .credentials(credentials)
        .build();

    let from = format!("{} <{}>", env.smtp_from_name, env.smtp_from_email)
        .parse::<Mailbox>()
        .map_err(|e| format!("Invalid from address: {}", e))?;
    let to = format!("{}", email.email_to)
        .parse::<Mailbox>()
        .map_err(|e| format!("Invalid to address: {}", e))?;

    let msg = MessageBuilder::new()
        .from(from)
        .to(to)
        .subject(email.email_subject.clone())
        .body(email.email_body.clone())
        .map_err(|e| format!("Failed to build email: {}", e))?;

    mailer
        .send(msg)
        .await
        .map_err(|e| format!("Failed to send email via SMTP: {}", e))?;

    Ok(())
}

async fn send_via_sendgrid(env: &service_utils::Env, email: &Email) -> Result<(), String> {
    let sg = SGClient::new(env.sendgrid_api_key.as_str());
    let mail_info = Mail::new()
        .add_to(Destination {
            address: email.email_to.as_str(),
            name: email.email_to.as_str(),
        })
        .add_from(email.email_from.as_str())
        .add_from_name(email.email_from_name.as_str())
        .add_subject(email.email_subject.as_str())
        .add_html(email.email_body.as_str());

    sg.send(mail_info)
        .await
        .map_err(|e| format!("Failed to send email via SendGrid: {}", e))?;

    Ok(())
}

pub async fn count_emails_by_target_id(
    env: &service_utils::Env,
    pool: &deadpool_postgres::Pool,
    request: Request<Empty>,
) -> Result<Response<Count>, Status> {
    let start = std::time::Instant::now();
    let metadata = request.metadata();
    let user = service_utils::auth(metadata, &env.jwt_secret)?;

    let conn = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get connection: {:?}", e);
        Status::internal("Failed to get connection")
    })?;

    let count = crate::email_db::count_emails_by_target_id(&conn, &user.email)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count emails: {:?}", e);
            Status::internal("Failed to count emails")
        })?;

    tracing::info!("count_emails_by_target_id: {:?}", start.elapsed());
    Ok(Response::new(Count { count }))
}

pub async fn get_emails_by_target_id(
    env: &service_utils::Env,
    pool: &deadpool_postgres::Pool,
    request: Request<Page>,
) -> Result<Response<ReceiverStream<Result<crate::proto::Email, Status>>>, Status> {
    let start = std::time::Instant::now();
    let metadata = request.metadata();
    let user = service_utils::auth(metadata, &env.jwt_secret)?;

    let conn = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get connection: {:?}", e);
        Status::internal("Failed to get connection")
    })?;

    let page = request.into_inner();
    let emails_stream =
        crate::email_db::get_emails_by_target_id(&conn, &user.email, page.offset, page.limit)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get emails: {:?}", e);
                Status::internal("Failed to get emails")
            })?;

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    tokio::spawn(async move {
        futures_util::pin_mut!(emails_stream);
        while let Ok(Some(note)) = tokio_stream::StreamExt::try_next(&mut emails_stream).await {
            let tx = tx.clone();
            tokio::spawn(async move {
                let email: Email = match note.try_into() {
                    Ok(note) => note,
                    Err(e) => {
                        tracing::error!("Failed to get note: {:?}", e);
                        return;
                    }
                };
                if let Err(e) = tx.send(Ok(email)).await {
                    tracing::error!("Failed to send email: {:?}", e);
                }
            });
        }
        tracing::info!("get_emails_by_target_id: {:?}", start.elapsed());
    });
    Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
        rx,
    )))
}

pub async fn send_email(
    env: &service_utils::Env,
    pool: &deadpool_postgres::Pool,
    request: Request<Email>,
) -> Result<Response<Email>, Status> {
    let start = std::time::Instant::now();
    let metadata = request.metadata();
    service_utils::auth(metadata, &env.jwt_secret)?;

    let email = request.into_inner();
    crate::email_validation::Validation::validate(&email)?;

    let mut conn = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get connection: {:?}", e);
        Status::internal("Failed to get connection")
    })?;
    let tr = conn.transaction().await.map_err(|e| {
        tracing::error!("Failed to start transaction: {:?}", e);
        Status::internal("Failed to start transaction")
    })?;

    let email = crate::email_db::insert_email(&tr, &email.target_id, &email)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert email: {:?}", e);
            Status::internal("Failed to insert email")
        })?;

    let send_result = if is_smtp_configured(env) {
        tracing::info!("Sending email via SMTP: {}", env.smtp_host);
        send_via_smtp(env, &email).await
    } else {
        tracing::info!("Sending email via SendGrid");
        send_via_sendgrid(env, &email).await
    };

    if let Err(e) = send_result {
        tracing::error!("Failed to send email: {}", e);
        return Err(Status::internal("Failed to send email"));
    }

    tr.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {:?}", e);
        Status::internal("Failed to commit transaction")
    })?;

    tracing::info!("send_email: {:?}", start.elapsed());
    Ok(Response::new(email))
}