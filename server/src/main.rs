use axum::Router;

use crate::services::{ Services, WorkerSpec, WorkersAllocate };

pub mod database;
pub mod services;
mod routes;
pub mod base;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let mongodb_connection_string = std::env
        ::var("MONGODB_CONNECTION_STRING")
        .expect("MONGODB_CONNECTION_STRING must be set in .env file");
    let host = std::env::var("HOST").expect("HOST must be set in .env file");

    println!("Connecting to DB...");
    let koii_database = database::initialize(&mongodb_connection_string).await.unwrap();
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

    let app = Router::new().nest("/user", routes::user::routes(services, koii_database));
    let listener = tokio::net::TcpListener::bind(host.clone()).await.unwrap();

    println!("Hello, world (world here is {})! :3", host);

    axum::serve(listener, app).await.unwrap();
}
