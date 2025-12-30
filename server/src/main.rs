use std::{ env::args, sync::Arc };

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::HeaderValue,
    http::{ Method, header::{ AUTHORIZATION, CONTENT_TYPE } },
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::cors::CorsLayer;
use crate::{
    cache::Cache,
    middlewares::auth,
    store::Store,
    utils::{ jwt::Jwt, turnstile::Turnstile },
    workers::{ WorkerSpec, Workers, WorkersAllocate },
};

pub mod store;
pub mod workers;
mod routes;
pub mod middlewares;
pub mod base;
pub mod utils;
pub mod cache;
pub mod consts;

pub struct AppState {
    pub worker: Workers,
    pub store: Store,
    pub cache: Cache,
    pub jwt: Jwt,
    pub turnstile: Turnstile,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let host = std::env::var("HOST").expect("HOST must be set in .env file");

    println!("Initializing server state...");
    let app_state = Arc::new(AppState {
        worker: Workers::new(WorkersAllocate {
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
        store: store::initialize().await.unwrap(),
        cache: cache::initialize().await.unwrap(),
        jwt: utils::jwt::Jwt::new(),
        turnstile: Turnstile::default(),
    });

    let cors = CorsLayer::new()
        .allow_origin(
            tower_http::cors::AllowOrigin::predicate(|origin: &HeaderValue, _| {
                let origin = origin.as_bytes();
                origin == b"https://koii.space" ||
                    (origin.starts_with(b"https://") && origin.ends_with(b".koii.space"))
            })
        )
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true);

    let app = Router::new()
        .nest("/user", routes::user::routes(app_state.clone()))
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), auth::authorize))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(cors);

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
