use std::{ sync::Arc, time::Duration };

use axum::{ Router, routing::{ get, patch, post } };

use crate::{ AppState, middlewares::time };

mod create;
mod verify;
mod login;
mod totp;
mod passkey;
mod logout;
mod delete;

#[derive(Clone)]
pub struct AccountRoutesState {
    pub app: Arc<AppState>,
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    let state = AccountRoutesState {
        app: app_state,
    };

    Router::new()
        .route("/", post(create::handler).delete(delete::handler))
        .route("/verify", patch(verify::handler))
        .route("/login", post(login::handler))
        .route("/logout", get(logout::handler))
        .nest("/totp", totp::routes(state.clone()))
        .nest("/passkey", passkey::routes(state.clone()))
        .layer(axum::middleware::from_fn_with_state(Duration::from_millis(800), time::padding))
        .with_state(state)
}
