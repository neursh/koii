use axum::http::HeaderName;
use mongodb::bson;
use redis::{ AsyncCommands, aio::MultiplexedConnection };
use reqwest::header::SET_COOKIE;

use crate::{ consts::TOKEN_MAX_AGE, utils::cookies };

#[derive(Clone)]
pub struct TokenQuery {
    pub user_id: String,
    pub created_at: i64,
    pub secret: String,
}

/// Redis is clone-based, so this might be cheap idk.
#[derive(Clone)]
pub struct TokenCache {
    pub endpoint: MultiplexedConnection,
}

impl TokenCache {
    /// Add a valid token to a user, returning a `SET_COOKIE` header.
    pub async fn add(
        &mut self,
        query: TokenQuery
    ) -> Result<(HeaderName, String), redis::RedisError> {
        let key = format!("token:<{}>", query.user_id);
        let member = format!("{}.{}", query.created_at, query.secret);
        self.endpoint.sadd::<&str, &str, usize>(&key, &member).await?;

        Ok((
            SET_COOKIE,
            cookies::consturct("token", format!("{}.{}", query.user_id, member), TOKEN_MAX_AGE),
        ))
    }

    pub async fn authorize(&mut self, query: &TokenQuery) -> Result<bool, redis::RedisError> {
        let key = format!("token:<{}>", query.user_id);
        let member = format!("{}.{}", query.created_at, query.secret);

        if self.endpoint.sismember::<&str, &str, bool>(&key, &member).await? {
            let alive =
                bson::DateTime::now().timestamp_millis() - query.created_at <= TOKEN_MAX_AGE;
            if !alive {
                self.endpoint.srem::<&str, &str, usize>(&key, &member).await?;
            }

            return Ok(alive);
        }

        return Ok(false);
    }

    pub async fn extend_ttl(&mut self, query: &TokenQuery) -> Result<(), redis::RedisError> {
        let key = format!("token:<{}>", query.user_id);

        let member = format!("{}.{}", query.created_at, query.secret);
        self.endpoint.srem::<&str, String, usize>(&key, member).await?;

        let member = format!("{}.{}", query.secret, bson::DateTime::now().timestamp_millis());
        self.endpoint.sadd::<&str, String, usize>(&key, member).await?;
        Ok(())
    }

    pub async fn delete_one(&mut self, query: &TokenQuery) -> Result<usize, redis::RedisError> {
        let key = format!("token:<{}>", query.user_id);
        let member = format!("{}.{}", query.created_at, query.secret);
        return Ok(self.endpoint.srem::<&str, String, usize>(&key, member).await?);
    }

    pub async fn delete_all(&mut self, user_id: &str) -> Result<usize, redis::RedisError> {
        let key = format!("token:<{}>", user_id);
        return Ok(self.endpoint.del::<&str, usize>(&key).await?);
    }
}
