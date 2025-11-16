use std::time::Duration;

use mongodb::{ IndexModel, bson, options::IndexOptions };
use serde::{ Deserialize, Serialize };

use crate::base::session::REFRESH_MAX_AGE;

#[derive(Clone, Deserialize, Serialize)]
pub struct RefreshDocument {
    pub user_id: String,
    pub expire_stamp: i64,
}

#[derive(Clone)]
pub struct RefreshStore {
    endpoint: mongodb::Collection<RefreshDocument>,
}
impl RefreshStore {
    pub async fn default(
        endpoint: mongodb::Collection<RefreshDocument>
    ) -> Result<Self, mongodb::error::Error> {
        endpoint.create_index(
            IndexModel::builder()
                .keys(bson::doc! { "expire_stamp": 1 })
                .options(
                    IndexOptions::builder()
                        .expire_after(Duration::from_secs(REFRESH_MAX_AGE as u64))
                        .build()
                )
                .build()
        ).await?;

        Ok(RefreshStore {
            endpoint,
        })
    }

    pub async fn add(&self, document: RefreshDocument) -> Result<(), mongodb::error::Error> {
        self.endpoint.insert_one(document).await?;
        Ok(())
    }

    /// Check if the refresh token is valid, the entry then deletes, ensuring that
    /// refresh token can only be used once.
    pub async fn permit(
        &self,
        user_id: &str,
        expire_stamp: i64
    ) -> Result<bool, mongodb::error::Error> {
        let item = self.endpoint.find_one_and_delete(
            bson::doc! { "user_id": user_id, "expire_stamp": expire_stamp }
        ).await?;
        Ok(item.is_some())
    }
}
