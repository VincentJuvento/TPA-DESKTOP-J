use once_cell::sync::OnceCell;
use sqlx::{postgres::PgPoolOptions, PgPool};
use redis::Client as RedisClient;

static DB_POOL: OnceCell<PgPool> = OnceCell::new();
static REDIS_CLIENT: OnceCell<RedisClient> = OnceCell::new();

pub async fn init_db(database_url: &str) -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    DB_POOL.set(pool).ok();
    Ok(())
}

pub fn get_db() -> &'static PgPool {
    DB_POOL.get().expect("Database pool not initialized")
}

pub async fn run_migrations() -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../migrations").run(get_db()).await
}

pub fn init_redis(redis_url: &str) -> Result<(), redis::RedisError> {
    let client = RedisClient::open(redis_url)?;
    REDIS_CLIENT.set(client).ok();
    Ok(())
}

pub fn get_redis() -> &'static RedisClient {
    REDIS_CLIENT.get().expect("Redis client not initialized")
}
