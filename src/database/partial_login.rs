use mongodb::{ Collection, IndexModel, bson, error::WriteFailure, options::IndexOptions };
use serde::{ Deserialize, Serialize };

use crate::env::PARTIAL_LOGIN_MAX_AGE;

#[derive(Deserialize, Serialize)]
pub struct PartialLoginDocument {
    /// Unique ID to the account.
    pub account_id: String,
    pub identifier: String,
    pub created_at: bson::DateTime,
}

pub struct PartialLoginOperations {
    collection: Collection<PartialLoginDocument>,
}

impl PartialLoginOperations {
    pub async fn new(
        collection: Collection<PartialLoginDocument>
    ) -> Result<Self, mongodb::error::Error> {
        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "account_id": 1, "identifier": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build()
        ).await?;

        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "created_at": 1 })
                .options(IndexOptions::builder().expire_after(*PARTIAL_LOGIN_MAX_AGE).build())
                .build()
        ).await?;

        Ok(PartialLoginOperations { collection })
    }

    pub async fn consume(
        &self,
        document: &PartialLoginDocument
    ) -> Result<bool, mongodb::error::Error> {
        match self.collection.insert_one(document).await {
            Ok(_) => {}
            Err(error) => {
                match *error.kind {
                    mongodb::error::ErrorKind::Write(WriteFailure::WriteError(ref write_error)) if
                        write_error.code == 11000
                    => {
                        return Ok(false);
                    }
                    _ => {
                        return Err(error);
                    }
                }
            }
        }

        Ok(true)
    }
}
