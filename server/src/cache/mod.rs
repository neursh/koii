pub mod refresh;
pub mod verify;

use crate::cache::{ refresh::RefreshCache, verify::VerifyCache };

pub struct Cache {
    pub refresh: RefreshCache,
    pub verify: VerifyCache,
}

pub async fn initialize() -> Result<Cache, redis::RedisError> {
    let redis_host = std::env::var("REDIS_HOST").expect("REDIS_HOST must be set in .env file");

    let redis_client = redis::Client::open(redis_host)?.get_multiplexed_async_connection().await?;

    Ok(Cache {
        refresh: RefreshCache { endpoint: redis_client.clone() },
        verify: VerifyCache { endpoint: redis_client.clone() },
    })
}
