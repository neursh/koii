use redis::{ AsyncCommands, aio::MultiplexedConnection };

use crate::consts::REFRESH_MAX_AGE;

pub struct RefreshCacheQuery {
    pub user_id: String,
    pub created_at: i64,
}

/// Redis is clone-based, so this might be cheap idk.
#[derive(Clone)]
pub struct RefreshCache {
    pub endpoint: MultiplexedConnection,
}

impl RefreshCache {
    pub async fn add(&mut self, query: RefreshCacheQuery) -> Result<(), redis::RedisError> {
        let key = format!("<{}>::jwt::refresh::{}", query.user_id, query.created_at);
        self.endpoint.set::<&str, bool, bool>(&key, true).await?;
        self.endpoint.expire::<&str, bool>(&key, REFRESH_MAX_AGE).await?;
        Ok(())
    }

    /// Check if the refresh token is valid, the entry then deletes, ensuring that
    /// refresh token can only be used once.
    pub async fn permit(&mut self, query: RefreshCacheQuery) -> Result<bool, redis::RedisError> {
        let key = format!("<{}>::jwt::refresh::{}", query.user_id, query.created_at);
        return Ok(self.endpoint.get_del::<&str, Option<bool>>(&key).await?.is_some());
    }
}
