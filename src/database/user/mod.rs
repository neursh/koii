use redis::aio::MultiplexedConnection;

use crate::database::user::{ token::TokenOperations, document::DocumentOperations };

pub mod document;
pub mod token;

pub struct UserDatabase {
    pub document: DocumentOperations,
    pub token: TokenOperations,
}

impl UserDatabase {
    pub async fn default(
        mongo_database: mongodb::Database,
        redis_client: MultiplexedConnection
    ) -> Result<Self, mongodb::error::Error> {
        let users_collection = mongo_database.collection("users");
        let tokens_collection = mongo_database.collection("tokens");

        Ok(UserDatabase {
            document: DocumentOperations::new(users_collection).await.unwrap(),
            token: TokenOperations::new(tokens_collection, redis_client.clone()).await.unwrap(),
        })
    }
}
