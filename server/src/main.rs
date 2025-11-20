use std::{ env::args, sync::Arc };

use axum::{ Router, extract::DefaultBodyLimit, http::HeaderValue, routing::get };
use axum_server::tls_rustls::RustlsConfig;
use tower_http::cors::CorsLayer;
use crate::{
    database::KoiiDatabase,
    services::{ Services, WorkerSpec, WorkersAllocate },
    utils::{ jwt::Jwt, cookie_query },
};

pub mod database;
pub mod services;
mod routes;
pub mod base;
pub mod utils;

pub struct AppState {
    pub services: Services,
    pub database: KoiiDatabase,
    pub jwt: Jwt,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let host = std::env::var("HOST").expect("HOST must be set in .env file");

    println!("Initializing server state...");
    let app_state = Arc::new(AppState {
        services: Services::new(WorkersAllocate {
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
        }),
        database: database::initialize().await.unwrap(),
        jwt: utils::jwt::Jwt::new(),
    });

    let cors = CorsLayer::new()
        .allow_origin(
            tower_http::cors::AllowOrigin::predicate(|origin: &HeaderValue, _| {
                let origin = origin.as_bytes();
                origin == b"https://koii.space" ||
                    (origin.starts_with(b"https://") && origin.ends_with(b".koii.space"))
            })
        )
        .allow_credentials(true);

    let app = Router::new()
        .nest("/user", routes::user::routes(app_state.clone()))
        .route(
            "/",
            get(async || "hi")
        )
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), cookie_query::authorize))
        .layer(cors)
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024));

    println!("Hello, world (world here is {})! :3", host);

    let mode: Vec<String> = args().collect();
    match mode[1].as_str() {
        "online" => {
            println!("Serving in online mode...");
            rustls::crypto::ring::default_provider().install_default().unwrap();
            let tls_config = RustlsConfig::from_pem_file(
                "cf-ocert.pem",
                "cf-okey.pem"
            ).await.unwrap();
            axum_server
                ::bind_rustls(host.parse().unwrap(), tls_config)
                .serve(app.into_make_service()).await
                .unwrap();
        }
        "offline" => {
            println!("Serving in offline mode...");
            axum_server::bind(host.parse().unwrap()).serve(app.into_make_service()).await.unwrap();
        }
        _ => panic!("Invalid mode."),
    }
}
