use mongodb::{ Collection, IndexModel, bson, options::IndexOptions };
use redis::{ AsyncCommands, RedisError, aio::MultiplexedConnection };
use serde::{ Deserialize, Serialize };
use thiserror::Error;

use crate::{ consts::REFRESH_MAX_AGE, utils::jwt::TokenClaims };

#[derive(Deserialize, Serialize)]
pub struct TokenDocument {
    /// Unique ID to the user.
    pub user_id: String,

    /// The token's identifier.
    pub identifier: String,

    /// TTL: REFRESH_MAX_AGE
    pub created_at: bson::DateTime,
}

#[derive(Error, Debug)]
pub enum TokenOperationError {
    #[error("Bad database")] Database(#[from] mongodb::error::Error),
    #[error("Bad bson")] Bson(#[from] mongodb::bson::error::Error),
    #[error("Bad cache")] Cache(#[from] RedisError),
}

#[derive(Clone)]
pub struct TokenOperations {
    collection: Collection<TokenDocument>,
    cache: MultiplexedConnection,
}
impl TokenOperations {
    pub async fn new(
        collection: Collection<TokenDocument>,
        cache: MultiplexedConnection
    ) -> Result<Self, TokenOperationError> {
        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "created_at": 1 })
                .options(IndexOptions::builder().expire_after(REFRESH_MAX_AGE).build())
                .build()
        ).await?;

        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "user_id": 1, "identifier": 1 })
                .build()
        ).await?;

        Ok(TokenOperations { collection, cache })
    }

    /// Create and add token to cache and database.
    ///
    /// Returns a pair of valid JWT token using the current time on server.
    ///
    /// Formatted as cookies can be passed to client.
    pub async fn create(
        &mut self,
        user_id: String,
        identifier: String,
        exp: u64
    ) -> Result<(), TokenOperationError> {
        let cache_key = format!("user:{}:token:{}", user_id, identifier);

        // Preload cache.
        self.cache.set::<&str, bool, String>(&cache_key, true).await?;
        self.cache.expire_at::<&str, bool>(&cache_key, exp as i64).await?;

        // Add database entry as a fallback.
        self.collection.insert_one(TokenDocument {
            user_id,
            identifier,
            created_at: bson::DateTime::from_millis(
                ((exp - REFRESH_MAX_AGE.as_secs()) * 1000) as i64
            ),
        }).await?;

        Ok(())
    }

    pub async fn authorize(&mut self, claims: &TokenClaims) -> Result<bool, TokenOperationError> {
        let status = self.cache.get::<String, Option<bool>>(
            format!("user:{}:token:{}", claims.user_id, claims.identifier)
        ).await?;

        return match status {
            Some(status) => Ok(status),
            None => self.refetch(claims).await,
        };
    }

    pub async fn revoke(&mut self, claims: &TokenClaims) -> Result<bool, TokenOperationError> {
        self.cache.set::<String, bool, String>(
            format!("user:{}:token:{}", &claims.user_id, &claims.identifier),
            false
        ).await?;

        let db_result = self.collection.delete_one(
            bson::doc! { "user_id": &claims.user_id, "identifier": &claims.identifier }
        ).await?;

        Ok(db_result.deleted_count == 1)
    }

    pub async fn revoke_all(&mut self, user_id: &str) -> Result<u64, TokenOperationError> {
        let mut tokens_cursor = self.collection.find(bson::doc! { "user_id": user_id }).await?;

        // Loop through database to batch a cache request for all tokens.
        let mut mset_props: Vec<(String, bool)> = Vec::new();
        while tokens_cursor.advance().await? {
            let token_doc: TokenDocument = bson::deserialize_from_slice(
                tokens_cursor.current().as_bytes()
            )?;

            mset_props.push((
                format!("user:{}:token:{}", &token_doc.user_id, &token_doc.identifier),
                false,
            ));
        }

        let db_result = self.collection.delete_many(bson::doc! { "user_id": user_id }).await?;

        self.cache.mset::<_, _, String>(&mset_props).await?;

        Ok(db_result.deleted_count)
    }

    /// Cache miss, ask the database instead if this token is valid or not.
    async fn refetch(&mut self, claims: &TokenClaims) -> Result<bool, TokenOperationError> {
        let document = self.collection.find_one(
            bson::doc! { "_id": &claims.user_id, "identifier": &claims.identifier }
        ).await?;

        let cache_key = format!("user:{}:token:{}", claims.user_id, claims.identifier);

        let status = match document {
            Some(_) => {
                self.cache.set::<&str, bool, String>(&cache_key, true).await?;
                Ok(true)
            }
            None => {
                self.cache.set::<&str, bool, String>(&cache_key, false).await?;
                Ok(false)
            }
        };

        self.cache.expire_at::<&str, bool>(&cache_key, claims.exp as i64).await?;

        status
    }
}
