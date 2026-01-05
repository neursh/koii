pub mod refresh;

use crate::cache::refresh::RefreshCache;

pub struct Cache {
    pub refresh: RefreshCache,
}

pub async fn initialize() -> Result<Cache, redis::RedisError> {
    tracing::info!(target: "redis_connector", "Connecting to redis host...");
    let redis_host = std::env::var("REDIS_HOST").expect("REDIS_HOST must be set in .env file");
    let redis_client = redis::Client::open(redis_host)?.get_multiplexed_async_connection().await?;

    Ok(Cache {
        refresh: RefreshCache { endpoint: redis_client.clone() },
    })
}
