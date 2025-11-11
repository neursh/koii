use std::time::Duration;

use mongodb::{ IndexModel, bson::{ self, Document }, options::IndexOptions };
use serde::{ Deserialize, Serialize };

#[derive(Clone, Deserialize, Serialize)]
pub struct UserDocument {
    pub email: String,
    pub password_hash: String,
    pub _id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_requested: Option<bson::DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_token: Option<String>,
}

#[derive(Clone)]
pub struct UsersCollection {
    collection: mongodb::Collection<UserDocument>,
}
impl UsersCollection {
    pub async fn default(
        collection: mongodb::Collection<UserDocument>
    ) -> Result<Self, mongodb::error::Error> {
        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "email": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build()
        ).await?;

        collection.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "verify_requested": 1 })
                .options(
                    IndexOptions::builder()
                        .expire_after(Duration::from_secs(10 * 60))
                        .build()
                )
                .build()
        ).await?;

        Ok(UsersCollection {
            collection,
        })
    }

    pub async fn add(&self, document: UserDocument) -> Result<(), mongodb::error::Error> {
        self.collection.insert_one(document).await?;
        Ok(())
    }

    pub async fn verify(&self, verify_token: String) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.update_one(
            bson::doc! { "verify_token": verify_token },
            bson::doc! { "$unset": { "verify_requested": "", "verify_token" : "" } }
        ).await?;
        if result.modified_count == 1 {
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn get_one(
        &self,
        query: Document
    ) -> Result<Option<UserDocument>, mongodb::error::Error> {
        self.collection.find_one(query).await
    }

    pub async fn delete(&self, id: String) -> Result<(), mongodb::error::Error> {
        self.collection.delete_one(bson::doc! { "_id": id }).await?;
        Ok(())
    }
}
