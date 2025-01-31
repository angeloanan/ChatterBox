use std::env;

use sqlx::migrate::MigrateDatabase;
use sqlx::{prelude::*, query_as};
use sqlx::{query, sqlite::SqlitePoolOptions, Pool, Sqlite};

#[derive(Debug, Clone, FromRow)]
pub struct Message {
    pub id: i64,
    pub username: String,
    pub message: String,
    pub created_at: sqlx::types::time::OffsetDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct Username {
    pub ip: i64,
    pub username: String,
}

#[tracing::instrument]
pub async fn init_db() -> Pool<Sqlite> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    if !Sqlite::database_exists(&database_url)
        .await
        .unwrap_or(false)
    {
        println!("Database doesn't exist yet. Creating database...");
        Sqlite::create_database(&database_url)
            .await
            .expect("Unable to create database. Exiting...")
    }

    SqlitePoolOptions::new()
        .connect(database_url.as_str())
        .await
        .expect("Unable to connect to database")
}

#[tracing::instrument]
pub async fn assert_table(connection: &Pool<Sqlite>) {
    query!(
        r#"
        CREATE TABLE IF NOT EXISTS
            messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                username TEXT NOT NULL,
                message TEXT NOT NULL,
                created_at DATETIME NOT NULL
            )
        "#
    )
    .execute(connection)
    .await
    .expect("Unable to create messages table");

    query!(
        r#"
        CREATE TABLE IF NOT EXISTS
            usernames (
                ip INTEGER PRIMARY KEY NOT NULL,
                username TEXT NOT NULL
            )
        "#
    )
    .execute(connection)
    .await
    .expect("Unable to create messages table");
}

pub async fn fetch_past_messages(connection: &Pool<Sqlite>) -> Vec<Message> {
    query_as::<_, Message>(
        r#"
        SELECT
            *
        FROM
            messages
        ORDER BY
            created_at DESC
        "#,
    )
    .fetch_all(connection)
    .await
    .expect("Unable to fetch past messages")
}

pub async fn add_message(connection: &Pool<Sqlite>, username: &str, message: &str, time: &i64) {
    let nanotime = time * 1_000;

    query_as!(
        Message,
        r#"
        INSERT INTO
            messages (username, message, created_at)
        VALUES
            (?, ?, ?)
        "#,
        username,
        message,
        nanotime
    )
    .execute(connection)
    .await
    .expect("Unable to add message");
}

/// Clean up database by removing messages older than 7 days
pub async fn cleanup_message(connection: &Pool<Sqlite>) {
    query_as!(
        Message,
        r#"
        DELETE FROM messages
        WHERE
            created_at < DATETIME('now', '-7 days')
        "#
    )
    .execute(connection)
    .await
    .expect("Unable to clean up database");
}

// pub async fn fetch_username(connection: &Pool<Sqlite>, ip: i64) {
//     query_as!(
//         Username,
//         r#"
//         SELECT
//             *
//         FROM
//             usernames
//         WHERE
//             ip = ?
//         "#,
//         ip
//     )
//     .fetch_one(connection)
//     .await
//     .expect("Unable to fetch username")
// }
//
// pub async fn store_username(connection: &Pool<Sqlite>, ip: i64, username: &str) {
//     query_as!(
//         Username,
//         r#"
//         INSERT INTO
//             usernames (ip, username)
//         VALUES
//             (?, ?)
//         "#,
//         ip,
//         username
//     )
//     .execute(connection)
//     .await
//     .expect("Unable to store username");
// }
