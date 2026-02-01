use std::{ sync::Arc, time::{ Duration, SystemTime } };

use axum::{
    Router,
    extract::Request,
    middleware::Next,
    response::IntoResponse,
    routing::{ get, patch, post },
};

use crate::AppState;

mod create;
mod verify;
mod login;
mod tfa;
mod logout;
mod delete;

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
        .nest("/2fa", tfa::routes(state.clone()))
        .layer(axum::middleware::from_fn(time_raiser))
        .with_state(state)
}

const RAISE_MAX_TO: Duration = Duration::from_millis(800);

pub async fn time_raiser(request: Request, next: Next) -> impl IntoResponse {
    let start = SystemTime::now();
    next.run(request).await;

    match start.elapsed() {
        Ok(finish) => {
            if finish < RAISE_MAX_TO {
                tokio::time::sleep(RAISE_MAX_TO - finish).await;
            }
        }
        Err(_) => {}
    }
}
