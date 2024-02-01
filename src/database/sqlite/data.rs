use chrono::Utc;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use tracing::error;

use crate::anilist_struct::run::minimal_anime::ActivityData;
use crate::constant::DATA_SQLITE_DB;
use crate::database::sqlite::pool::get_sqlite_pool;
use crate::database_struct::server_activity_struct::{ServerActivity, ServerActivityFull};
use crate::database_struct::user_color_struct::UserColor;
use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::Error::SqlInsertError;

/// Inserts or replaces a record in the `ping_history` table of a SQLite database.
///
/// The function takes a 'shard_id' and a 'latency', both of type `String`, as input.
/// It attempts to insert or replace a record with these values into the `ping_history` table.
/// The `shard_id` and `latency` are most likely related to a latency reported for a specific shard ID.
/// The current timestamp is also stored with each record.
/// The function is asynchronous and returns nothing.
///
/// # Arguments
///
/// * `shard_id` - A String containing the ID of a shard.
/// * `latency` - A String containing the latency value.
///
/// # Errors
///
/// This function will log errors encountered when executing the SQL command, but does not return them.
pub async fn set_data_ping_history_sqlite(
    shard_id: String,
    latency: String,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let now = Utc::now().timestamp().to_string();
    sqlx::query("INSERT OR REPLACE INTO ping_history (shard_id, timestamp, ping) VALUES (?, ?, ?)")
        .bind(shard_id)
        .bind(now)
        .bind(latency)
        .execute(&pool)
        .await
        .map_err(|e| {
            Error(SqlInsertError(format!(
                "Failed to insert into the table. {}",
                e
            )))
        })?;
    pool.close().await;
    Ok(())
}

/// This function retrieves language data for a guild from a SQLite database.
///
/// # Arguments
///
/// * `guild_id` - A string representing the ID of the guild.
///
/// # Returns
///
/// A tuple containing the language and guild ID as optional strings.
/// If the data is found in the database, it will be returned.
/// If not found, both values will be `None`.
pub async fn get_data_guild_langage_sqlite(
    guild_id: String,
) -> Result<(Option<String>, Option<String>), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let row: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT lang, guild FROM guild_lang WHERE guild = ?")
            .bind(guild_id)
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None));
    pool.close().await;
    Ok(row)
}

/// Sets the language for a guild in the SQLite database.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild.
/// * `lang_struct` - The language to set for the guild.
pub async fn set_data_guild_langage_sqlite(
    guild_id: &String,
    lang: &String,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    sqlx::query("INSERT OR REPLACE INTO guild_lang (guild, lang) VALUES (?, ?)")
        .bind(guild_id)
        .bind(lang)
        .execute(&pool)
        .await
        .map_err(|e| {
            Error(SqlInsertError(format!(
                "Failed to insert into the table. {}",
                e
            )))
        })?;
    pool.close().await;
    Ok(())
}

/// Retrieves activity data from SQLite database.
///
/// # Returns
///
/// A `Vec<ActivityData>` containing the retrieved data.
///
pub async fn get_data_activity_sqlite(now: String) -> Result<Vec<ActivityData>, AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let rows: Vec<ActivityData> = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id, webhook, episode, name, delays, image FROM activity_data WHERE timestamp = ?",
    )
        .bind(now)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    Ok(rows)
}

/// Sets data activity in SQLite database.
///
/// # Arguments
///
/// * `anime_id` - The ID of the anime.
/// * `timestamp` - The timestamp.
/// * `guild_id` - The ID of the guild.
/// * `webhook` - The webhook URL.
/// * `episode` - The episode number.
/// * `name` - The name of the anime.
/// * `delays` - The delays.
///
pub async fn set_data_activity_sqlite(
    server_activity_full: ServerActivityFull,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    sqlx::query(
        "INSERT OR REPLACE INTO activity_data (anime_id, timestamp, server_id, webhook, episode, name, delays, image) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
        .bind(server_activity_full.anime_id)
        .bind(server_activity_full.timestamp)
        .bind(server_activity_full.guild_id)
        .bind(server_activity_full.webhook)
        .bind(server_activity_full.episode)
        .bind(server_activity_full.name)
        .bind(server_activity_full.delays)
        .bind(server_activity_full.image)
        .execute(&pool)
        .await.map_err(|e| Error(SqlInsertError(format!("Failed to insert into the table. {}", e))))?;
    pool.close().await;
    Ok(())
}

pub async fn get_data_module_activation_status_sqlite(
    guild_id: &String,
) -> Result<(Option<String>, Option<bool>, Option<bool>, Option<bool>), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let row: (Option<String>, Option<bool>, Option<bool>, Option<bool>) = sqlx::query_as(
        "SELECT guild_id, ai_module, anilist_module, game_module FROM module_activation WHERE guild = ?",
    )
        .bind(guild_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None, None, None));
    pool.close().await;
    Ok(row)
}

pub async fn set_data_module_activation_status_sqlite(
    guild_id: &String,
    anilist_value: bool,
    ai_value: bool,
    game_value: bool,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let _ = sqlx::query(
        "INSERT OR REPLACE INTO module_activation (guild_id, anilist_module, ai_module, game_module) VALUES (?, ?, ?)",
    )
        .bind(guild_id)
        .bind(anilist_value)
        .bind(ai_value)
        .bind(game_value)
        .execute(&pool)
        .await
        .map_err(|e| Error(SqlInsertError(format!("Failed to insert into the table. {}", e))))?;
    pool.close().await;
    Ok(())
}

pub async fn remove_data_activity_status_sqlite(
    server_id: String,
    anime_id: String,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let _ = sqlx::query("DELETE FROM activity_data WHERE anime_id = ? AND server_id = ?")
        .bind(anime_id)
        .bind(server_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            Error(SqlInsertError(format!(
                "Failed to insert into the table. {}",
                e
            )))
        })?;
    pool.close().await;

    Ok(())
}

pub async fn get_data_module_activation_kill_switch_status_sqlite(
) -> Result<(Option<String>, Option<bool>, Option<bool>, Option<bool>), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let row: (Option<String>, Option<bool>, Option<bool>, Option<bool>) = sqlx::query_as(
        "SELECT id, ai_module, anilist_module, game_module FROM module_activation WHERE guild = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or((None, None, None, None));
    pool.close().await;

    Ok(row)
}

pub async fn get_one_activity_sqlite(
    server_id: String,
    anime_id: i32,
) -> Result<(Option<String>, Option<String>, Option<String>), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let row: (Option<String>, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT anime_id, timestamp, server_id FROM activity_data WHERE anime_id = ? AND server_id = ?",
    )
        .bind(anime_id)
        .bind(server_id)
        .fetch_one(&pool)
        .await
        .unwrap_or((None, None, None));

    pool.close().await;

    Ok(row)
}

pub async fn get_registered_user_sqlite(
    user_id: &String,
) -> Result<(Option<String>, Option<String>), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let row: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT anilist_id, user_id FROM registered_user WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None));
    pool.close().await;

    Ok(row)
}

pub async fn set_registered_user_sqlite(
    user_id: &String,
    username: &String,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    sqlx::query_as("INSERT OR REPLACE INTO registered_user (user_id, anilist_id) VALUES (?, ?)")
        .bind(user_id)
        .bind(username)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            Error(SqlInsertError(format!(
                "Failed to insert into the table. {}",
                e
            )))
        })?;
    pool.close().await;

    Ok(())
}

pub async fn set_user_approximated_color_sqlite(
    user_id: &String,
    color: &String,
    pfp_url: &String,
    image: &String,
) -> Result<(), AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let _ = sqlx::query(
        "INSERT OR REPLACE INTO user_color (user_id, color, pfp_url, image) VALUES (?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(color)
    .bind(pfp_url)
    .bind(image)
    .execute(&pool)
    .await
    .map_err(|e| {
        Error(SqlInsertError(format!(
            "Failed to insert into the table. {}",
            e
        )))
    })?;
    pool.close().await;

    Ok(())
}

pub async fn get_user_approximated_color_sqlite(user_id: &String) -> Result<UserColor, AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let row: UserColor =
        sqlx::query_as("SELECT user_id, color, pfp_url, image FROM user_color WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap_or(UserColor {
                user_id: None,
                color: None,
                pfp_url: None,
                image: None,
            });
    pool.close().await;

    Ok(row)
}

pub async fn get_all_server_activity_sqlite(
    server_id: &String,
) -> Result<Vec<ServerActivity>, AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let rows: Vec<ServerActivity> = sqlx::query_as(
        "SELECT
       anime_id,
       timestamp,
       server_id,
       webhook,
       episode,
       name,
       delays
       FROM activity_data WHERE server_id = ?
   ",
    )
    .bind(server_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    //.map_err(|_| SqlSelectError(String::from("Failed to select from the table.")))?;

    Ok(rows)
}

pub async fn get_all_user_approximated_color_sqlite() -> Result<Vec<UserColor>, AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let row: Vec<UserColor> =
        sqlx::query_as("SELECT user_id, color, pfp_url, image FROM user_color")
            .fetch_all(&pool)
            .await
            .unwrap_or(vec![UserColor {
                user_id: None,
                color: None,
                pfp_url: None,
                image: None,
            }]);
    pool.close().await;

    Ok(row)
}

pub async fn get_data_activity_with_server_and_anime_id_sqlite(
    anime_id: &String,
    server_id: &String,
) -> Result<Option<String>, AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let row = sqlx::query_as(
        "SELECT
       webhook
       FROM activity_data WHERE server_id = ? and anime_id = ?
   ",
    )
    .bind(server_id)
    .bind(anime_id)
    .fetch_one(&pool)
    .await
    .unwrap_or_default();
    pool.close().await;
    Ok(row)
}

pub async fn get_data_all_activity_by_server_sqlite(
    server_id: &String,
) -> Result<Vec<(String, String)>, AppError> {
    let pool = get_sqlite_pool(DATA_SQLITE_DB).await?;
    let rows = sqlx::query_as(
        "SELECT
           anime_id, name
           FROM activity_data WHERE server_id = ?
       ",
    )
    .bind(server_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    pool.close().await;

    Ok(rows)
}
