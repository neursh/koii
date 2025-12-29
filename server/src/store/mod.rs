use crate::store::{ users::UsersStore };

pub mod users;

pub struct Store {
    pub users: UsersStore,
}

pub async fn initialize() -> Result<Store, mongodb::error::Error> {
    let mongodb_connection_string = std::env
        ::var("MONGODB_CONNECTION_STRING")
        .expect("MONGODB_CONNECTION_STRING must be set in .env file");

    let mongo_client = mongodb::Client::with_uri_str(mongodb_connection_string).await?;
    let mongo_database = mongo_client.database("koii");

    Ok(Store {
        users: UsersStore::default(mongo_database.collection("users")).await?,
    })
}
