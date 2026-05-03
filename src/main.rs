use std::{ env::args, net::SocketAddr, sync::Arc };

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::HeaderValue,
    http::{ Method, header::{ AUTHORIZATION, CONTENT_TYPE } },
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::{ cors::CorsLayer, trace::TraceLayer };
use crate::{
    database::Database,
    utils::{ jwt::Jwt, turnstile::Turnstile },
    workers::{ WorkerSpec, Workers, WorkersAllocate },
};

pub mod database;
pub mod workers;
mod routes;
pub mod middlewares;
pub mod base;
pub mod utils;
pub mod consts;

pub struct AppState {
    pub worker: Workers,
    pub db: Database,
    pub jwt: Jwt,
    pub turnstile: Turnstile,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt().init();

    rustls::crypto::ring::default_provider().install_default().unwrap();

    let host = std::env
        ::var("HOST")
        .expect("HOST must be set in .env file")
        .parse::<SocketAddr>()
        .unwrap();

    tracing::info!("Initializing server state...");
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
        db: Database::default().await.unwrap(),
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
        .nest("/account", routes::account::routes(app_state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(1 * 1024 * 1024))
        .layer(cors);

    tracing::info!("Hello, world (world here is {})! :3", host);

    let mode: Vec<String> = args().collect();
    if mode.len() != 2 {
        tracing::error!("No required arguments provided. [secure/insecure]");
        return;
    }

    match mode[1].as_str() {
        "secure" => {
            tracing::info!("Serving in secure context...");
            let tls_config = RustlsConfig::from_pem_file(
                "cf-ocert.pem",
                "cf-okey.pem"
            ).await.unwrap();
            axum_server
                ::bind_rustls(host, tls_config)
                .serve(app.into_make_service()).await
                .unwrap();
        }
        "insecure" => {
            tracing::info!("Serving in insecure context...");
            tracing::warn!(
                "Insecure context is for local development only, do not fuck this up, plwease QwQ"
            );
            axum_server::bind(host).serve(app.into_make_service()).await.unwrap();
        }
        _ => tracing::error!("No context chosen, shutting down... [secure/insecure]"),
    }
}
