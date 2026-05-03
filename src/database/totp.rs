use mongodb::{ Collection, IndexModel, bson, error::WriteFailure, options::IndexOptions };
use serde::{ Deserialize, Serialize };

use crate::utils::totp::Totp;

#[derive(Deserialize, Serialize)]
pub struct TotpDocument {
    /// Unique ID to the account.
    pub account_id: String,
    pub totp: Totp,
}

pub struct TotpOperations {
    collection: Collection<TotpDocument>,
}

impl TotpOperations {
    pub async fn new(collection: Collection<TotpDocument>) -> Result<Self, mongodb::error::Error> {
        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "account_id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build()
        ).await?;

        Ok(TotpOperations { collection })
    }

    pub async fn add(&self, document: &TotpDocument) -> Result<bool, mongodb::error::Error> {
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

    pub async fn get(&self, account_id: &str) -> Result<Option<Totp>, mongodb::error::Error> {
        let totp_collection = self.collection.find_one(
            bson::doc! { "account_id": account_id }
        ).await?;

        match totp_collection {
            Some(totp_collection) => Ok(Some(totp_collection.totp)),
            None => Ok(None),
        }
    }

    pub async fn delete(&self, account_id: &str) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.delete_one(bson::doc! { "account_id": account_id }).await?;

        Ok(result.deleted_count == 1)
    }
}
