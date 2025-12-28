use crate::store::{ refresh::RefreshStore, users::UsersStore };

pub mod users;
pub mod refresh;

pub struct Store {
    pub users: UsersStore,
    pub refresh: RefreshStore,
}

pub async fn initialize() -> Result<Store, mongodb::error::Error> {
    let mongodb_connection_string = std::env
        ::var("MONGODB_CONNECTION_STRING")
        .expect("MONGODB_CONNECTION_STRING must be set in .env file");

    let mongo_client = mongodb::Client::with_uri_str(mongodb_connection_string).await?;
    let mongo_database = mongo_client.database("koii");

    let user_space = mongo_database.collection("users");
    let refresh_space = mongo_database.collection("refresh");
    Ok(Store {
        users: UsersStore::default(user_space).await.unwrap(),
        refresh: RefreshStore::default(refresh_space).await.unwrap(),
    })
}
