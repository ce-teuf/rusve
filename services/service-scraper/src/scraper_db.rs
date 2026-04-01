use anyhow::Result;
use deadpool_postgres::Object;
use time::format_description::well_known::Iso8601;
use uuid::Uuid;

use crate::proto::{Item, Job, JobResponse, Source};

// ── TryFrom impls ────────────────────────────────────────────

impl TryFrom<tokio_postgres::Row> for Source {
    type Error = anyhow::Error;

    fn try_from(row: tokio_postgres::Row) -> Result<Self, Self::Error> {
        let id: Uuid = row.try_get("id")?;
        let created: time::OffsetDateTime = row.try_get("created")?;
        let updated: time::OffsetDateTime = row.try_get("updated")?;
        let field_rules: serde_json::Value = row.try_get("field_rules")?;

        Ok(Source {
            id: id.to_string(),
            created: created.format(&Iso8601::DEFAULT)?.to_string(),
            updated: updated.format(&Iso8601::DEFAULT)?.to_string(),
            name: row.try_get("name")?,
            source_url: row.try_get("source_url")?,
            source_type: row.try_get("source_type")?,
            integration_mode: row.try_get("integration_mode")?,
            auto_schedule: row.try_get("auto_schedule")?,
            field_rules: field_rules.to_string(),
            active: row.try_get("active")?,
        })
    }
}

impl TryFrom<tokio_postgres::Row> for Job {
    type Error = anyhow::Error;

    fn try_from(row: tokio_postgres::Row) -> Result<Self, Self::Error> {
        let id: Uuid = row.try_get("id")?;
        let created: time::OffsetDateTime = row.try_get("created")?;
        let updated: time::OffsetDateTime = row.try_get("updated")?;
        let source_id: Option<Uuid> = row.try_get("source_id")?;
        let item_count: i32 = row.try_get("item_count")?;

        Ok(Job {
            id: id.to_string(),
            created: created.format(&Iso8601::DEFAULT)?.to_string(),
            updated: updated.format(&Iso8601::DEFAULT)?.to_string(),
            source_id: source_id.map(|u| u.to_string()).unwrap_or_default(),
            source_url: row.try_get("source_url")?,
            source_type: row.try_get("source_type")?,
            status: row.try_get("status")?,
            item_count: item_count as i64,
            error: row.try_get("error")?,
        })
    }
}

impl TryFrom<tokio_postgres::Row> for Item {
    type Error = anyhow::Error;

    fn try_from(row: tokio_postgres::Row) -> Result<Self, Self::Error> {
        let id: Uuid = row.try_get("id")?;
        let created: time::OffsetDateTime = row.try_get("created")?;
        let updated: time::OffsetDateTime = row.try_get("updated")?;
        let job_id: Uuid = row.try_get("job_id")?;
        let raw_data: serde_json::Value = row.try_get("raw_data")?;
        let validation_errors: serde_json::Value = row.try_get("validation_errors")?;
        let pushed_at: Option<time::OffsetDateTime> = row.try_get("pushed_at")?;

        Ok(Item {
            id: id.to_string(),
            created: created.format(&Iso8601::DEFAULT)?.to_string(),
            updated: updated.format(&Iso8601::DEFAULT)?.to_string(),
            job_id: job_id.to_string(),
            raw_data: raw_data.to_string(),
            validation_status: row.try_get("validation_status")?,
            validation_errors: validation_errors.to_string(),
            pushed_at: pushed_at
                .map(|t| t.format(&Iso8601::DEFAULT).unwrap_or_default().to_string())
                .unwrap_or_default(),
            pushed_target: row.try_get("pushed_target")?,
        })
    }
}

// ── Source queries ───────────────────────────────────────────

pub async fn list_sources(conn: &Object) -> Result<Vec<Source>> {
    let rows = conn
        .query(
            "select * from scrape_sources order by created desc",
            &[],
        )
        .await?;
    rows.into_iter().map(Source::try_from).collect()
}

pub async fn list_auto_sources(conn: &Object) -> Result<Vec<Source>> {
    let rows = conn
        .query(
            "select * from scrape_sources where integration_mode = 'AUTO' and active = true and auto_schedule != ''",
            &[],
        )
        .await?;
    rows.into_iter().map(Source::try_from).collect()
}

pub async fn get_source(conn: &Object, id: &str) -> Result<Source> {
    let row = conn
        .query_one("select * from scrape_sources where id = $1", &[&Uuid::parse_str(id)?])
        .await?;
    Source::try_from(row)
}

pub async fn insert_source(conn: &Object, s: &Source) -> Result<Source> {
    let field_rules: serde_json::Value = serde_json::from_str(&s.field_rules).unwrap_or(serde_json::json!([]));
    let row = conn
        .query_one(
            "insert into scrape_sources (id, name, source_url, source_type, integration_mode, auto_schedule, field_rules, active) \
             values ($1, $2, $3, $4, $5, $6, $7, $8) returning *",
            &[
                &Uuid::now_v7(),
                &s.name,
                &s.source_url,
                &s.source_type,
                &s.integration_mode,
                &s.auto_schedule,
                &field_rules,
                &s.active,
            ],
        )
        .await?;
    Source::try_from(row)
}

pub async fn update_source(conn: &Object, s: &Source) -> Result<Source> {
    let id = Uuid::parse_str(&s.id)?;
    let field_rules: serde_json::Value = serde_json::from_str(&s.field_rules).unwrap_or(serde_json::json!([]));
    let row = conn
        .query_one(
            "update scrape_sources set name=$1, source_url=$2, source_type=$3, \
             integration_mode=$4, auto_schedule=$5, field_rules=$6, active=$7 \
             where id=$8 returning *",
            &[
                &s.name,
                &s.source_url,
                &s.source_type,
                &s.integration_mode,
                &s.auto_schedule,
                &field_rules,
                &s.active,
                &id,
            ],
        )
        .await?;
    Source::try_from(row)
}

pub async fn delete_source(conn: &Object, id: &str) -> Result<()> {
    conn.execute(
        "delete from scrape_sources where id = $1",
        &[&Uuid::parse_str(id)?],
    )
    .await?;
    Ok(())
}

// ── Job queries ──────────────────────────────────────────────

pub async fn list_jobs(conn: &Object, offset: i64, limit: i64) -> Result<Vec<JobResponse>> {
    let rows = conn
        .query(
            "select j.*, coalesce(s.name, '') as source_name \
             from scrape_jobs j \
             left join scrape_sources s on j.source_id = s.id \
             order by j.created desc offset $1 limit $2",
            &[&offset, &limit],
        )
        .await?;

    rows.into_iter()
        .map(|row| {
            let source_name: String = row.try_get("source_name")?;
            let job = Job::try_from(row)?;
            Ok(JobResponse {
                job: Some(job),
                source_name,
            })
        })
        .collect()
}

pub async fn get_job(conn: &Object, id: &str) -> Result<Job> {
    let row = conn
        .query_one("select * from scrape_jobs where id = $1", &[&Uuid::parse_str(id)?])
        .await?;
    Job::try_from(row)
}

// ── Item queries ─────────────────────────────────────────────

pub async fn list_items(
    conn: &Object,
    job_id: &str,
    status: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<Item>> {
    let job_uuid = Uuid::parse_str(job_id)?;
    let rows = if status.is_empty() {
        conn.query(
            "select * from scrape_items where job_id = $1 order by created desc offset $2 limit $3",
            &[&job_uuid, &offset, &limit],
        )
        .await?
    } else {
        conn.query(
            "select * from scrape_items where job_id = $1 and validation_status = $2 order by created desc offset $3 limit $4",
            &[&job_uuid, &status, &offset, &limit],
        )
        .await?
    };
    rows.into_iter().map(Item::try_from).collect()
}

pub async fn update_item_status(
    conn: &Object,
    id: &str,
    status: &str,
    errors_json: &str,
) -> Result<Item> {
    let errors: serde_json::Value = serde_json::from_str(errors_json).unwrap_or(serde_json::json!([]));
    let row = conn
        .query_one(
            "update scrape_items set validation_status=$1, validation_errors=$2 where id=$3 returning *",
            &[&status, &errors, &Uuid::parse_str(id)?],
        )
        .await?;
    Item::try_from(row)
}

pub async fn get_pending_items_for_job(conn: &Object, job_id: &str) -> Result<Vec<Item>> {
    let rows = conn
        .query(
            "select * from scrape_items where job_id=$1 and validation_status='PENDING'",
            &[&Uuid::parse_str(job_id)?],
        )
        .await?;
    rows.into_iter().map(Item::try_from).collect()
}

pub async fn get_source_field_rules_by_job(conn: &Object, job_id: &str) -> Result<String> {
    let row = conn
        .query_opt(
            "select s.field_rules from scrape_jobs j \
             left join scrape_sources s on j.source_id = s.id \
             where j.id = $1",
            &[&Uuid::parse_str(job_id)?],
        )
        .await?;
    let field_rules: Option<serde_json::Value> = row.as_ref().and_then(|r| r.try_get("field_rules").ok());
    Ok(field_rules.map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string()))
}

pub async fn approve_all_valid(conn: &Object, job_id: &str) -> Result<i64> {
    let count = conn
        .execute(
            "update scrape_items set validation_status='APPROVED' where job_id=$1 and validation_status='VALID'",
            &[&Uuid::parse_str(job_id)?],
        )
        .await?;
    Ok(count as i64)
}

pub async fn get_approved_items_for_job(conn: &Object, job_id: &str) -> Result<Vec<Item>> {
    let rows = conn
        .query(
            "select * from scrape_items where job_id=$1 and validation_status='APPROVED'",
            &[&Uuid::parse_str(job_id)?],
        )
        .await?;
    rows.into_iter().map(Item::try_from).collect()
}

pub async fn get_job_source_type(conn: &Object, job_id: &str) -> Result<String> {
    let row = conn
        .query_one("select source_type from scrape_jobs where id=$1", &[&Uuid::parse_str(job_id)?])
        .await?;
    Ok(row.try_get("source_type")?)
}

pub async fn mark_items_pushed(conn: &Object, ids: &[String], target: &str) -> Result<()> {
    let uuids: Vec<Uuid> = ids
        .iter()
        .filter_map(|id| Uuid::parse_str(id).ok())
        .collect();
    conn.execute(
        "update scrape_items set validation_status='PUSHED', pushed_at=now(), pushed_target=$1 \
         where id = any($2)",
        &[&target, &uuids],
    )
    .await?;
    Ok(())
}

// ── Auto-push (called by scheduler) ─────────────────────────

pub async fn get_valid_items_by_source(conn: &Object, source_id: &str) -> Result<Vec<(Item, String)>> {
    let rows = conn
        .query(
            "select si.*, sj.source_type as job_source_type \
             from scrape_items si \
             join scrape_jobs sj on si.job_id = sj.id \
             where sj.source_id = $1 and si.validation_status = 'VALID'",
            &[&Uuid::parse_str(source_id)?],
        )
        .await?;

    rows.into_iter()
        .map(|row| {
            let source_type: String = row.try_get("job_source_type")?;
            let item = Item::try_from(row)?;
            Ok((item, source_type))
        })
        .collect()
}

// ── db_data queries ──────────────────────────────────────────

pub async fn insert_data_item(
    conn: &Object,
    source_type: &str,
    data_json: &str,
    scrape_item_id: &str,
) -> Result<()> {
    let data: serde_json::Value = serde_json::from_str(data_json)
        .unwrap_or(serde_json::json!({}));
    conn.execute(
        "insert into data_items (id, source_type, data, scrape_item_id) values ($1, $2, $3, $4)",
        &[
            &Uuid::now_v7(),
            &source_type,
            &data,
            &Uuid::parse_str(scrape_item_id)?,
        ],
    )
    .await?;
    Ok(())
}

// ── Auto-push orchestration ──────────────────────────────────

pub async fn auto_push_source(
    source_id: &str,
    pool: &deadpool_postgres::Pool,
    data_pool: &deadpool_postgres::Pool,
) -> Result<i64> {
    let conn = pool.get().await?;
    let items_with_type = get_valid_items_by_source(&conn, source_id).await?;

    if items_with_type.is_empty() {
        return Ok(0);
    }

    let data_conn = data_pool.get().await?;
    let mut ids = Vec::with_capacity(items_with_type.len());

    for (item, source_type) in &items_with_type {
        insert_data_item(&data_conn, source_type, &item.raw_data, &item.id).await?;
        ids.push(item.id.clone());
    }

    mark_items_pushed(&conn, &ids, "db_data").await?;

    tracing::info!(
        "Auto-push: {} items pushed for source {}",
        ids.len(),
        source_id
    );

    Ok(ids.len() as i64)
}
