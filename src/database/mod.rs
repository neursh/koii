use crate::database::account::AccountDatabase;

pub mod account;

pub struct Database {
    pub account: AccountDatabase,
}

impl Database {
    pub async fn default() -> Self {
        tracing::info!("Connecting to mongodb host...");
        let mongodb_connection_string = std::env
            ::var("MONGODB_CONNECTION_STRING")
            .expect("MONGODB_CONNECTION_STRING must be set in .env file");

        let mongo_client = mongodb::Client::with_uri_str(mongodb_connection_string).await.unwrap();
        let mongo_database = mongo_client.database("koii");

        tracing::info!("Connecting to redis host...");
        let redis_host = std::env::var("REDIS_HOST").expect("REDIS_HOST must be set in .env file");
        let redis_client = redis::Client
            ::open(redis_host)
            .unwrap()
            .get_multiplexed_async_connection().await
            .unwrap();

        Database {
            account: AccountDatabase::default(mongo_database, redis_client).await.unwrap(),
        }
    }
}
