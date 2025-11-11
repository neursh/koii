use std::env;

use crate::services::{ Services, WorkerSpec, WorkersAllocate };

mod database;
mod services;
mod conductor_app;
mod http_app;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let mongodb_connection_string = std::env
        ::var("MONGODB_CONNECTION_STRING")
        .expect("MONGODB_CONNECTION_STRING must be set in .env file");
    let http_host = std::env::var("HTTP_HOST").expect("HTTP_HOST must be set in .env file");
    let conductor_host = std::env
        ::var("CONDUCTOR_HOST")
        .expect("CONDUCTOR_HOST must be set in .env file");

    let koii_collections = database::initialize(&mongodb_connection_string).await.unwrap();

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

    println!("Hello, world!");
}
