use mongodb::{ Collection, IndexModel, bson, options::{ FindOneOptions, IndexOptions } };
use serde::{ Deserialize, Serialize };

use crate::{ consts::{ EMAIL_VERIFY_EXPIRE, USER_DELETE_FRAME }, utils::totp::Totp };

#[derive(Deserialize, Serialize)]
pub struct UserDocument {
    /// Unique ID to the user.
    pub user_id: String,

    /// User's email.
    pub email: String,

    /// User's password hash using argon2id.
    pub password_hash: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp: Option<Totp>,

    /// The time when user verified the account locking in as the creation time.
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

    /// Mark the user as deleted when the user request for deletion.
    ///
    /// TTL: USER_DELETE_FRAME
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bson::DateTime>,
}

pub struct DocumentOperations {
    collection: Collection<UserDocument>,
}
impl DocumentOperations {
    pub async fn new(collection: Collection<UserDocument>) -> Result<Self, mongodb::error::Error> {
        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "email": 1, "user_id": 1 })
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
                .options(IndexOptions::builder().expire_after(USER_DELETE_FRAME).build())
                .build()
        ).await?;

        Ok(DocumentOperations { collection })
    }

    pub async fn add(&self, document: &UserDocument) -> Result<(), mongodb::error::Error> {
        self.collection.insert_one(document).await?;
        Ok(())
    }

    pub async fn get(
        &self,
        filter: bson::Document
    ) -> Result<Option<UserDocument>, mongodb::error::Error> {
        self.collection.find_one(filter).await
    }

    pub async fn exists(&self, filter: bson::Document) -> Result<bool, mongodb::error::Error> {
        Ok(self.get(filter).await?.is_some())
    }

    pub async fn add_totp(
        &self,
        user_id: &str,
        user_totp: &Totp
    ) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.update_one(
            bson::doc! { "user_id": user_id, "totp": { "$exists": false } },
            bson::doc! { "$set": { "totp": bson::serialize_to_bson(user_totp).unwrap() } }
        ).await?;

        Ok(result.modified_count == 1)
    }

    pub async fn get_totp(&self, user_id: &str) -> Result<Option<Totp>, mongodb::error::Error> {
        let partial_document = self.collection
            .find_one(bson::doc! { "user_id": user_id })
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

    pub async fn mark_deletion(&self, user_id: &str) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.update_one(
            bson::doc! { "user_id": user_id },
            bson::doc! { "$set": { "deleted": bson::DateTime::now() } }
        ).await?;

        Ok(result.modified_count == 1)
    }

    pub async fn unmark_deletion(&self, user_id: &str) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.update_one(
            bson::doc! { "user_id": user_id },
            bson::doc! { "$unset": { "deleted": "" } }
        ).await?;

        Ok(result.modified_count == 1)
    }
}
