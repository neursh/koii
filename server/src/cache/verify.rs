use redis::{ AsyncCommands, aio::MultiplexedConnection };

use crate::consts::EMAIL_VERIFY_EXPIRE;

pub struct VerifyCacheQuery {
    pub user_id: String,
    pub code: String,
}

/// Redis is clone-based, so this might be cheap idk.
#[derive(Clone)]
pub struct VerifyCache {
    pub endpoint: MultiplexedConnection,
}

impl VerifyCache {
    pub async fn add(&mut self, query: VerifyCacheQuery) -> Result<(), redis::RedisError> {
        let key = format!("<{}>::user::verify", query.code);
        self.endpoint.set::<&str, String, String>(&key, query.user_id).await?;
        self.endpoint.expire::<&str, bool>(&key, EMAIL_VERIFY_EXPIRE).await?;
        Ok(())
    }

    /// Check if the refresh token is valid, the entry then deletes, ensuring that
    /// refresh token can only be used once.
    pub async fn permit(&mut self, code: &str) -> Result<Option<String>, redis::RedisError> {
        let key = format!("<{}>::user::verify", code);
        return self.endpoint.get_del::<&str, Option<String>>(&key).await;
    }
}
