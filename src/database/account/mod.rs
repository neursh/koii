use redis::aio::MultiplexedConnection;

use crate::database::account::{ token::TokenOperations, document::DocumentOperations };

pub mod document;
pub mod token;

pub struct AccountDatabase {
    pub document: DocumentOperations,
    pub token: TokenOperations,
}

impl AccountDatabase {
    pub async fn default(
        mongo_database: mongodb::Database,
        redis_client: MultiplexedConnection
    ) -> Result<Self, mongodb::error::Error> {
        let accounts_collection = mongo_database.collection("accounts");
        let tokens_collection = mongo_database.collection("tokens");

        Ok(AccountDatabase {
            document: DocumentOperations::new(accounts_collection).await.unwrap(),
            token: TokenOperations::new(tokens_collection, redis_client.clone()).await.unwrap(),
        })
    }
}
