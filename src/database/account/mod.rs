use redis::aio::MultiplexedConnection;

use crate::database::account::{
    document::DocumentOperations,
    sudo::SudoOperations,
    token::TokenOperations,
};

pub mod document;
pub mod token;
pub mod sudo;

pub struct AccountDatabase {
    pub document: DocumentOperations,
    pub token: TokenOperations,
    pub sudo: SudoOperations,
}

impl AccountDatabase {
    pub async fn default(
        mongo_database: mongodb::Database,
        redis_client: MultiplexedConnection
    ) -> Result<Self, mongodb::error::Error> {
        let accounts_collection = mongo_database.collection("accounts");
        let tokens_collection = mongo_database.collection("tokens");
        let sudo_collection = mongo_database.collection("sudo");

        Ok(AccountDatabase {
            document: DocumentOperations::new(accounts_collection).await.unwrap(),
            token: TokenOperations::new(tokens_collection, redis_client.clone()).await.unwrap(),
            sudo: SudoOperations::new(sudo_collection).await.unwrap(),
        })
    }
}
