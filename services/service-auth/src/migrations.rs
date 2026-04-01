use anyhow::Result;

pub async fn run_migrations(pool: &deadpool_postgres::Pool) -> Result<()> {
    let client = pool.get().await?;
    client
        .batch_execute(
            r#"
            create table if not exists verifiers (
                id uuid primary key,
                created timestamptz not null default now(),
                csrf_token text not null,
                pkce_verifier text not null,
                unique (csrf_token, pkce_verifier)
            );

            create table if not exists local_credentials (
                id uuid primary key,
                created timestamptz not null default now(),
                updated timestamptz not null default now(),
                email text unique not null,
                password_hash text not null,
                verified bool not null default false
            );

            create table if not exists verification_codes (
                id uuid primary key,
                created timestamptz not null default now(),
                email text not null,
                code text not null,
                expires_at timestamptz not null
            );
        "#,
        )
        .await?;

    Ok(())
}
