use std::time::Duration;

use mongodb::{ IndexModel, bson::{ self, DateTime, Document }, options::IndexOptions };
use serde::{ Deserialize, Serialize };

#[derive(Deserialize, Serialize)]
pub struct UserDocument {
    /// User's email.
    pub email: String,

    /// User's password hash using argon2id.
    pub password_hash: String,

    /// The assigned unique ID to that user.
    pub _id: String,

    /// The time when the verify key was sent, this value gets deleted when
    /// the account was verifed. (TTL: 10 minutes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_requested: Option<bson::DateTime>,

    /// The actual verify code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_code: Option<String>,

    /// The time when user verified the account, locking in as the creation time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<bson::DateTime>,

    /// Invalidate any refresh tokens before this date.
    ///
    /// This value should only be changed when an account-wide logout is issued.
    /// For example: Password reset, lockdown mode. Things that would require extra verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_refresh_after: Option<bson::DateTime>,
}

pub struct UsersStore {
    endpoint: mongodb::Collection<UserDocument>,
}
impl UsersStore {
    pub async fn default(
        endpoint: mongodb::Collection<UserDocument>
    ) -> Result<Self, mongodb::error::Error> {
        endpoint.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "email": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build()
        ).await?;

        endpoint.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "verify_requested": 1 })
                .options(
                    IndexOptions::builder()
                        .expire_after(Duration::from_secs(10 * 60))
                        .build()
                )
                .build()
        ).await?;

        Ok(UsersStore {
            endpoint,
        })
    }

    pub async fn add(&self, document: UserDocument) -> Result<(), mongodb::error::Error> {
        self.endpoint.insert_one(document).await?;
        Ok(())
    }

    /// Verify user from the token sent via email.
    ///
    /// When user found and updated, `_id` will be returned, allowing verify to create a jwt.
    pub async fn verify(
        &self,
        verify_code: String
    ) -> Result<Option<String>, mongodb::error::Error> {
        if let Some(target) = self.get_one(bson::doc! { "verify_code": &verify_code }).await? {
            let result = self.endpoint.update_one(
                bson::doc! { "verify_code": &verify_code },
                bson::doc! {
                "$set": {
                    "created_at": DateTime::now(),
                },
                "$unset": {
                    "verify_requested": "",
                    "verify_code" : ""
                }
            }
            ).await?;
            if result.modified_count == 1 {
                return Ok(Some(target._id));
            }
        }

        Ok(None)
    }

    pub async fn get_one(
        &self,
        query: Document
    ) -> Result<Option<UserDocument>, mongodb::error::Error> {
        self.endpoint.find_one(query).await
    }

    pub async fn refresh_check() {}

    pub async fn delete(&self, id: String) -> Result<bool, mongodb::error::Error> {
        Ok(self.endpoint.delete_one(bson::doc! { "_id": id }).await?.deleted_count == 1)
    }
}
