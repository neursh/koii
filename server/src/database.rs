use crate::database::users::UsersCollection;

pub mod users;

#[derive(Clone)]
pub struct KoiiDatabase {
    pub users: UsersCollection,
}

pub async fn initialize(
    mongodb_connection_string: &str
) -> Result<KoiiDatabase, mongodb::error::Error> {
    let client = mongodb::Client::with_uri_str(mongodb_connection_string).await?;

    let database = client.database("koii");

    Ok(KoiiDatabase {
        users: UsersCollection::default(database.collection("users")).await.unwrap(),
    })
}
