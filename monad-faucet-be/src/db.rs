use std::time::Duration;

use sqlx::{Pool, Postgres};
use tokio::time::timeout;

/// Initialize the database connection pool with a timeout of 15 seconds
pub async fn init_db(db_url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    let pool = timeout(
        Duration::from_secs(15),
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(db_url),
    )
    .await
    .expect("Failed to connect to DB")
    .expect("Failed to connect to DB");

    // Check database health
    sqlx::query("SELECT 1").execute(&pool).await?;

    Ok(pool)
}
//TODO: run migrations when starting the server
