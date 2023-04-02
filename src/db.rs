use std::env;

use sqlx::types::time;
use sqlx::{prelude::*, query_as};
use sqlx::{query, sqlite::SqlitePoolOptions, Pool, Sqlite};

#[derive(Debug)]
struct Message {
    id: i64,
    username: String,
    message: String,
    created_at: sqlx::types::time::OffsetDateTime,
}

pub async fn init_db() -> Pool<Sqlite> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    SqlitePoolOptions::new()
        .connect(database_url.as_str())
        .await
        .expect("Unable to connect to database")
}

pub async fn assert_table(connection: &Pool<Sqlite>) {
    query!(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            username TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL
        )
        "#
    )
    .execute(connection)
    .await
    .expect("Unable to create table");
}

// FIXME: DATETIME errors?
// pub async fn fetch_past_messages(connection: &Pool<Sqlite>) -> Vec<Message> {
//     query_as!(
//         Message,
//         r#"
//         SELECT * FROM messages
//         ORDER BY created_at DESC
//         "#
//     )
//     .fetch_all(connection)
//     .await
//     .expect("Unable to fetch past messages")
// }

pub async fn add_message(connection: &Pool<Sqlite>, username: &str, message: &str) {
    let current_time = time::OffsetDateTime::now_utc();

    query_as!(
        Message,
        r#"
        INSERT INTO messages (username, message, created_at)
        VALUES (?, ?, ?)
        "#,
        username,
        message,
        current_time
    )
    .execute(connection)
    .await
    .expect("Unable to add message");
}
