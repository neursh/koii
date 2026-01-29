use std::sync::Arc;

use axum::{ Router, routing::{ patch, post, get } };

use crate::AppState;

pub mod create;
pub mod verify;
pub mod login;
pub mod logout;
pub mod delete;

#[derive(Clone)]
pub struct UserRoutesState {
    pub app: Arc<AppState>,
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    let state = UserRoutesState {
        app: app_state,
    };
    Router::new()
        .route("/", post(create::handler).delete(delete::handler))
        .route("/verify", patch(verify::handler))
        .route("/login", post(login::handler))
        .route("/logout", get(logout::handler))
        .with_state(state)
}
