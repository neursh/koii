use axum::{ Router, extract::DefaultBodyLimit, http::HeaderValue };
use tower_http::cors::CorsLayer;
use crate::{
    services::{ Services, WorkerSpec, WorkersAllocate },
    utils::middlewares::{ self, AuthorizationState },
};

pub mod database;
pub mod services;
mod routes;
pub mod base;
pub mod utils;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let host = std::env::var("HOST").expect("HOST must be set in .env file");

    println!("Connecting to DB...");
    let koii_database = database::initialize().await.unwrap();
    println!("DB connection established.");

    let services = Services::new(WorkersAllocate {
        // Allocate a reasonable amount of workers for password services.
        // This be using 100% when full load on all workers.
        // Password hashing is heavy after all.
        hash_pass: WorkerSpec {
            threads: 12,
            buffer: 2048,
        },
        verify_pass: WorkerSpec {
            threads: 12,
            buffer: 2048,
        },
        verify_email: WorkerSpec {
            threads: 1,
            buffer: 100,
        },
    });

    let jwt = utils::jwt::Jwt::new();

    let cors = CorsLayer::new().allow_origin(
        "https://*.koii.space".parse::<HeaderValue>().unwrap()
    );
    let app = Router::new()
        .nest("/user", routes::user::routes(services, koii_database.clone(), jwt.clone()))
        .layer(
            axum::middleware::from_fn_with_state(
                AuthorizationState {
                    jwt,
                    refresh_store: koii_database.refresh,
                },
                middlewares::authorize
            )
        )
        .layer(cors)
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024));
    let listener = tokio::net::TcpListener::bind(host.clone()).await.unwrap();

    println!("Hello, world (world here is {})! :3", host);

    axum::serve(listener, app).await.unwrap();
}
