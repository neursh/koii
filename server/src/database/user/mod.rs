use async_trait::async_trait;
use mongodb::{ IndexModel, bson, options::IndexOptions };
use redis::aio::MultiplexedConnection;

use crate::{ consts::EMAIL_VERIFY_EXPIRE, database::user::document::UserDocument };

pub mod document;

pub struct UserDatabase {
    document: mongodb::Collection<UserDocument>,
    sudo: MultiplexedConnection,
}

#[async_trait]
pub trait UserDocumentOperations {
    async fn add_user(&self, document: &UserDocument) -> Result<(), mongodb::error::Error>;

    async fn get_user(
        &self,
        filter: bson::Document
    ) -> Result<Option<UserDocument>, mongodb::error::Error>;

    async fn user_exists(&self, filter: bson::Document) -> Result<bool, mongodb::error::Error>;

    /// Verify user from the token sent via email.
    async fn verify_user(&self, verify_code: String) -> Result<bool, mongodb::error::Error>;
}

#[async_trait]
pub trait UserTokenOperations {
    async fn add_token(&self, user_id: String) -> Result<(), redis::RedisError>;

    async fn authorize(&self, token: String) -> Result<(), redis::RedisError>;

    async fn revoke(&self, token: String) -> Result<(), redis::RedisError>;
}

#[async_trait]
pub trait UserSudoOperations {
    /// Create a special token that will allow or show sensitive changes with TTL set by `SUDO_MAX_AGE`.
    async fn create_sudo(&self, user_id: &str) -> Result<String, redis::RedisError>;

    /// Authorize the sudo token given by user's request.
    async fn authorize_sudo(&self, user_id: &str, token: &str) -> Result<bool, redis::RedisError>;

    /// Force delete the sudo token, skipping TTL.
    async fn destroy_sudo(&self, user_id: &str, token: &str) -> Result<(), redis::RedisError>;
}

impl UserDatabase {
    pub async fn default(
        mongo_database: mongodb::Database,
        redis_client: MultiplexedConnection
    ) -> Result<Self, mongodb::error::Error> {
        let document = mongo_database.collection("user");
        document.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "email": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build()
        ).await?;
        document.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "verify_requested": 1 })
                .options(IndexOptions::builder().expire_after(EMAIL_VERIFY_EXPIRE).build())
                .build()
        ).await?;

        Ok(UserDatabase {
            document,
            sudo: redis_client.clone(),
        })
    }
}
