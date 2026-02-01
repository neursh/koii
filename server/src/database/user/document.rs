use async_trait::async_trait;
use mongodb::bson;
use serde::{ Deserialize, Serialize };

use crate::{ database::user::{ UserDatabase, UserDocumentOperations }, utils::totp::Totp };

#[derive(Deserialize, Serialize)]
pub struct UserDocument {
    /// Unique ID to the user.
    #[serde(rename = "_id")]
    pub id: String,

    /// User's email.
    pub email: String,

    /// User's password hash using argon2id.
    pub password_hash: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp: Option<Totp>,

    /// The time when user verified the account, locking in as the creation time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<bson::DateTime>,

    /// The time when the verify key was sent, this value gets deleted when
    /// the account was verifed. (TTL: 10 minutes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_requested: Option<bson::DateTime>,

    /// The actual verify code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_code: Option<String>,
}

#[async_trait]
impl UserDocumentOperations for UserDatabase {
    async fn add_user(&self, document: &UserDocument) -> Result<(), mongodb::error::Error> {
        self.document.insert_one(document).await?;
        Ok(())
    }

    async fn get_user(
        &self,
        filter: bson::Document
    ) -> Result<Option<UserDocument>, mongodb::error::Error> {
        self.document.find_one(filter).await
    }

    async fn user_exists(&self, filter: bson::Document) -> Result<bool, mongodb::error::Error> {
        Ok(self.get_user(filter).await?.is_some())
    }

    async fn verify_user(&self, verify_code: String) -> Result<bool, mongodb::error::Error> {
        let result = self.document.update_one(
            bson::doc! { "verify_code": &verify_code },
            bson::doc! {
                "$set": {
                    "created_at": bson::DateTime::now(),
                },
                "$unset": {
                    "verify_requested": "",
                    "verify_code" : ""
                }
            }
        ).await?;
        if result.modified_count == 1 {
            return Ok(true);
        }

        Ok(false)
    }
}
