use redis::aio::MultiplexedConnection;

use crate::database::user::{ token::TokenOperations, user::UserOperations };

pub mod user;
pub mod token;

pub struct UserDatabase {
    pub document: UserOperations,
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
            document: UserOperations::new(users_collection).await.unwrap(),
            token: TokenOperations::new(tokens_collection, redis_client.clone()).await.unwrap(),
        })
    }
}
