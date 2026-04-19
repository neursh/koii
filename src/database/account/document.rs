use mongodb::{
    Collection,
    IndexModel,
    bson,
    options::{ CountOptions, FindOneOptions, IndexOptions },
};
use serde::{ Deserialize, Serialize };

use crate::{ consts::{ EMAIL_VERIFY_EXPIRE, ACCOUNT_DELETE_FRAME }, utils::totp::Totp };

#[derive(Deserialize, Serialize)]
pub struct AccountDocument {
    /// Unique ID to the account.
    pub account_id: String,

    /// Account's email.
    pub email: String,

    /// Account's password hash using argon2id.
    pub password_hash: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp: Option<Totp>,

    /// The time when the user verified the account locking in as the creation time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<bson::DateTime>,

    /// The time when the verify key was sent, this value gets deleted when
    /// the account was verifed.
    ///
    /// TTL: EMAIL_VERIFY_EXPIRE
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_requested: Option<bson::DateTime>,

    /// The actual verify code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_code: Option<String>,

    /// Mark the account as deleted when the user request for deletion.
    ///
    /// TTL: ACCOUNT_DELETE_FRAME
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bson::DateTime>,
}

pub struct DocumentOperations {
    collection: Collection<AccountDocument>,
}
impl DocumentOperations {
    pub async fn new(
        collection: Collection<AccountDocument>
    ) -> Result<Self, mongodb::error::Error> {
        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "email": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build()
        ).await?;

        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "account_id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build()
        ).await?;

        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "verify_requested": 1 })
                .options(IndexOptions::builder().expire_after(EMAIL_VERIFY_EXPIRE).build())
                .build()
        ).await?;

        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "deleted": 1 })
                .options(IndexOptions::builder().expire_after(ACCOUNT_DELETE_FRAME).build())
                .build()
        ).await?;

        Ok(DocumentOperations { collection })
    }

    pub async fn add(&self, document: &AccountDocument) -> Result<(), mongodb::error::Error> {
        self.collection.insert_one(document).await?;
        Ok(())
    }

    pub async fn get(
        &self,
        filter: bson::Document
    ) -> Result<Option<AccountDocument>, mongodb::error::Error> {
        self.collection.find_one(filter).await
    }

    pub async fn exists(&self, account_id: String) -> Result<bool, mongodb::error::Error> {
        let exists = self.collection
            .count_documents(bson::doc! { "account_id": account_id })
            .with_options(CountOptions::builder().limit(1).build()).await?;

        return Ok(exists == 1);
    }

    pub async fn add_totp(
        &self,
        account_id: &str,
        account_totp: &Totp
    ) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.update_one(
            bson::doc! { "account_id": account_id, "totp": { "$exists": false } },
            bson::doc! { "$set": { "totp": bson::serialize_to_bson(account_totp).unwrap() } }
        ).await?;

        Ok(result.modified_count == 1)
    }

    pub async fn get_totp(&self, account_id: &str) -> Result<Option<Totp>, mongodb::error::Error> {
        let partial_document = self.collection
            .find_one(bson::doc! { "account_id": account_id })
            .with_options(
                Some(
                    FindOneOptions::builder()
                        .projection(Some(bson::doc! { "totp": 1, "_id": 0 }))
                        .build()
                )
            ).await?;

        match partial_document {
            Some(partial_document) => {
                match partial_document.totp {
                    Some(totp) => Ok(Some(totp)),
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    pub async fn verify_email(&self, verify_code: &str) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.update_one(
            bson::doc! { "verify_code": verify_code },
            bson::doc! {
                "$set": {
                    "created_at": bson::DateTime::now()
                },
                "$unset": {
                    "verify_requested": "",
                    "verify_code" : ""
                }
            }
        ).await?;

        Ok(result.modified_count == 1)
    }

    pub async fn mark_deletion(&self, account_id: &str) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.update_one(
            bson::doc! { "account_id": account_id },
            bson::doc! { "$set": { "deleted": bson::DateTime::now() } }
        ).await?;

        Ok(result.modified_count == 1)
    }

    pub async fn unmark_deletion(&self, account_id: &str) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.update_one(
            bson::doc! { "account_id": account_id },
            bson::doc! { "$unset": { "deleted": "" } }
        ).await?;

        Ok(result.modified_count == 1)
    }
}
