use crate::database::postgresql::migration::migration_dispatch::migrate_postgres;
use crate::database::postgresql::pool::get_postgresql_pool;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{NotACommandError, SqlCreateError};
use crate::error_enum::NotACommandError::{
    NotACommandCreatingDatabaseError, NotACommandCreatingDatabaseFileError,
    NotACommandCreatingTableError, NotACommandGettingDatabaseFileError,
    NotACommandInsertingDatabaseError,
};
use sqlx::{Pool, Postgres};

pub async fn init_postgres() -> Result<(), AppError> {
    let pool = get_postgresql_pool().await?;
    init_postgres_cache(&pool).await?;
    pool.close().await;
    let pool = get_postgresql_pool().await?;
    init_postgres_data(&pool).await?;
    pool.close().await;
    migrate_postgres().await?;
    Ok(())
}

async fn init_postgres_cache(pool: &Pool<Postgres>) -> Result<(), AppError> {
    // Check if the database exists
    let exists: (bool, ) =
        sqlx::query_as("SELECT EXISTS (SELECT FROM pg_database WHERE datname = $1)")
            .bind("CACHE")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                NotACommandError(NotACommandGettingDatabaseFileError(format!(
                    "Failed to check if the database exists. {}",
                    e
                )))
            })?;

    // If the database does not exist, create it
    if !exists.0 {
        sqlx::query("CREATE DATABASE CACHE")
            .execute(pool)
            .await
            .map_err(|e| {
                NotACommandError(NotACommandCreatingDatabaseError(format!(
                    "Failed to create the database. {}",
                    e
                )))
            })?;
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS request_cache (
           json TEXT PRIMARY KEY,
           response TEXT NOT NULL,
           last_updated BIGINT NOT NULL
       )",
    )
        .execute(pool)
        .await
        .map_err(|e| {
            NotACommandError(NotACommandCreatingTableError(format!(
                "Failed to create the table. {}",
                e
            )))
        })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS cache_stats (
           key TEXT PRIMARY KEY,
           response TEXT NOT NULL,
           last_updated BIGINT NOT NULL,
           last_page BIGINT NOT NULL
       )",
    )
        .execute(pool)
        .await
        .map_err(|e| {
            NotACommandError(NotACommandCreatingTableError(format!(
                "Failed to create the table. {}",
                e
            )))
        })?;
    Ok(())
}

async fn init_postgres_data(pool: &Pool<Postgres>) -> Result<(), AppError> {
    // Check if the database exists
    let exists: (bool, ) =
        sqlx::query_as("SELECT EXISTS (SELECT FROM pg_database WHERE datname = $1)")
            .bind("DATA")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                NotACommandError(NotACommandGettingDatabaseFileError(format!(
                    "Failed to check if the database exists. {}",
                    e
                )))
            })?;

    // If the database does not exist, create it
    if !exists.0 {
        sqlx::query("CREATE DATABASE DATA")
            .execute(pool)
            .await
            .map_err(|e| {
                NotACommandError(NotACommandCreatingDatabaseError(format!(
                    "Failed to create the database. {}",
                    e
                )))
            })?;
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ping_history (
                    shard_id TEXT,
                    timestamp TEXT,
                    ping TEXT NOT NULL,
                    PRIMARY KEY (shard_id, timestamp)
                )",
    )
        .execute(pool)
        .await
        .map_err(|e| {
            NotACommandError(NotACommandCreatingTableError(format!(
                "Failed to create the table. {}",
                e
            )))
        })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS guild_lang (
            guild TEXT PRIMARY KEY,
            lang TEXT NOT NULL
        )",
    )
        .execute(pool)
        .await
        .map_err(|e| {
            NotACommandError(NotACommandCreatingTableError(format!(
                "Failed to create the table. {}",
                e
            )))
        })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS activity_data (
        anime_id TEXT,
        timestamp TEXT,
        server_id TEXT,
        webhook TEXT,
        episode TEXT,
        name TEXT,
        delays BIGINT DEFAULT 0,
        image TEXT,
        PRIMARY KEY (anime_id, server_id)
    )",
    )
        .execute(pool)
        .await
        .map_err(|e| {
            NotACommandError(NotACommandCreatingTableError(format!(
                "Failed to create the table. {}",
                e
            )))
        })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS module_activation (
       guild_id TEXT PRIMARY KEY,
       ai_module BIGINT,
       anilist_module BIGINT,
        game_module BIGINT
   )",
    )
        .execute(pool)
        .await
        .map_err(|e| {
            NotACommandError(NotACommandCreatingTableError(format!(
                "Failed to create the table. {}",
                e
            )))
        })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS registered_user  (
            user_id TEXT PRIMARY KEY,
            anilist_id TEXT
        )",
    )
        .execute(pool)
        .await
        .map_err(|e| {
            NotACommandError(NotACommandCreatingTableError(format!(
                "Failed to create the table. {}",
                e
            )))
        })?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS global_kill_switch (
            id TEXT PRIMARY KEY,
            ai_module BIGINT,
            anilist_module BIGINT,
            game_module BIGINT
        )",
    )
        .execute(pool)
        .await
        .map_err(|e| {
            NotACommandError(NotACommandCreatingTableError(format!(
                "Failed to create the table. {}",
                e
            )))
        })?;

    sqlx::query(
        "INSERT INTO global_kill_switch (id, anilist_module, ai_module, game_module) VALUES ($1, $2, $3, $4)
    ON CONFLICT (id) DO UPDATE SET anilist_module = excluded.anilist_module, ai_module = excluded.ai_module, game_module = excluded.game_module",
    )
        .bind("1")
        .bind(1)
        .bind(1)
        .bind(1)
        .execute(pool)
        .await.map_err(|e| NotACommandError(NotACommandInsertingDatabaseError(format!("Failed to create the database table. {}", e))))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user_color (
    user_id TEXT PRIMARY KEY,
    color TEXT NOT NULL,
    pfp_url TEXT NOT NULL,
    image TEXT NOT NULL
     )",
    )
        .execute(pool)
        .await
        .map_err(|e| {
            NotACommandError(NotACommandCreatingTableError(format!(
                "Failed to create the table. {}",
                e
            )))
        })?;

    Ok(())
}
