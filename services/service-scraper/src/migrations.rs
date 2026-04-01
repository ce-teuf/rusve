use anyhow::Result;

pub async fn run_migrations(pool: &deadpool_postgres::Pool) -> Result<()> {
    let client = pool.get().await?;
    client
        .batch_execute(
            r#"
            create or replace function trigger_set_timestamp ()
            returns trigger
            as $$
            begin
                new.updated = now();
                return new;
            end;
            $$ language plpgsql;

            create table if not exists scrape_sources (
                id               uuid primary key,
                created          timestamptz not null default now(),
                updated          timestamptz not null default now(),
                name             text not null,
                source_url       text not null unique,
                source_type      text not null,
                integration_mode text not null default 'MANUAL',
                auto_schedule    text not null default '',
                field_rules      jsonb not null default '[]',
                active           boolean not null default true
            );
            drop trigger if exists set_timestamp on scrape_sources;
            create trigger set_timestamp before update on scrape_sources
                for each row execute procedure trigger_set_timestamp();

            create table if not exists scrape_jobs (
                id          uuid primary key,
                created     timestamptz not null default now(),
                updated     timestamptz not null default now(),
                source_id   uuid references scrape_sources(id) on delete set null,
                source_url  text not null,
                source_type text not null,
                status      text not null default 'RUNNING',
                item_count  int not null default 0,
                error       text not null default ''
            );
            drop trigger if exists set_timestamp on scrape_jobs;
            create trigger set_timestamp before update on scrape_jobs
                for each row execute procedure trigger_set_timestamp();

            create table if not exists scrape_items (
                id                uuid primary key,
                created           timestamptz not null default now(),
                updated           timestamptz not null default now(),
                job_id            uuid not null references scrape_jobs(id) on delete cascade,
                raw_data          jsonb not null,
                validation_status text not null default 'PENDING',
                validation_errors jsonb not null default '[]',
                pushed_at         timestamptz,
                pushed_target     text not null default ''
            );
            drop trigger if exists set_timestamp on scrape_items;
            create trigger set_timestamp before update on scrape_items
                for each row execute procedure trigger_set_timestamp();
            "#,
        )
        .await?;
    Ok(())
}

pub async fn run_data_migrations(pool: &deadpool_postgres::Pool) -> Result<()> {
    let client = pool.get().await?;
    client
        .batch_execute(
            r#"
            create table if not exists data_items (
                id             uuid primary key,
                created        timestamptz not null default now(),
                source_type    text not null,
                data           jsonb not null,
                scrape_item_id uuid not null
            );
            "#,
        )
        .await?;
    Ok(())
}
