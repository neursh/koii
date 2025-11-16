use std::sync::Arc;

use axum::{ Router, routing::{ patch, post } };
use tokio::sync::Semaphore;

use crate::{ database::KoiiDatabase, services::Services, utils::jwt::Jwt };

pub mod create;
pub mod verify;
pub mod login;
pub mod delete;

#[derive(Clone)]
pub struct RoutesSemaphore {
    pub create: Arc<Semaphore>,
}

#[derive(Clone)]
pub struct RouteState {
    pub services: Services,
    pub koii_database: KoiiDatabase,
    pub jwt: Jwt,
    pub semaphores: RoutesSemaphore,
}

pub fn routes(services: Services, koii_database: KoiiDatabase, jwt: Jwt) -> Router {
    let state = RouteState {
        services,
        koii_database,
        jwt,
        semaphores: RoutesSemaphore { create: Arc::new(Semaphore::new(8)) },
    };
    Router::new()
        .route("/", post(create::handler).delete(delete::handler))
        .route("/verify", patch(verify::handler))
        .route("/login", post(login::handler))
        .with_state(state)
}
