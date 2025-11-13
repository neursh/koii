use crate::database::users::UsersStore;

pub mod users;

#[derive(Clone)]
pub struct KoiiDatabase {
    pub users: UsersStore,
}

pub async fn initialize() -> Result<KoiiDatabase, mongodb::error::Error> {
    let mongodb_connection_string = std::env
        ::var("MONGODB_CONNECTION_STRING")
        .expect("MONGODB_CONNECTION_STRING must be set in .env file");

    let mongo_client = mongodb::Client::with_uri_str(mongodb_connection_string).await?;
    let mongo_database = mongo_client.database("koii");

    let user_space = mongo_database.collection("users");
    Ok(KoiiDatabase {
        users: UsersStore::default(user_space).await.unwrap(),
    })
}
