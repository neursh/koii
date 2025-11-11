use crate::database::users::UsersCollection;

pub mod users;

#[derive(Clone)]
pub struct KoiiCollections {
    pub users: UsersCollection,
}

pub async fn initialize(
    mongodb_connection_string: &str
) -> Result<KoiiCollections, mongodb::error::Error> {
    let client = mongodb::Client::with_uri_str(mongodb_connection_string).await?;

    let database = client.database("kewar");

    Ok(KoiiCollections {
        users: UsersCollection::default(database.collection("users")).await.unwrap(),
    })
}
