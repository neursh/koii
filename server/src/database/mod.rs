use crate::database::user::UserDatabase;

pub mod user;

pub struct Database {
    pub user: UserDatabase,
}

impl Database {
    pub async fn default() -> Self {
        tracing::info!(target: "database.store", "Connecting to mongodb host...");
        let mongodb_connection_string = std::env
            ::var("MONGODB_CONNECTION_STRING")
            .expect("MONGODB_CONNECTION_STRING must be set in .env file");

        let mongo_client = mongodb::Client::with_uri_str(mongodb_connection_string).await.unwrap();
        let mongo_database = mongo_client.database("koii");

        tracing::info!(target: "database.cache", "Connecting to redis host...");
        let redis_host = std::env::var("REDIS_HOST").expect("REDIS_HOST must be set in .env file");
        let redis_client = redis::Client
            ::open(redis_host)
            .unwrap()
            .get_multiplexed_async_connection().await
            .unwrap();

        Database {
            user: UserDatabase::default(mongo_database, redis_client).await.unwrap(),
        }
    }
}
