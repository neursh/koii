use axum::Router;
use axum::routing::post;

use crate::{ routes::user::UserRoutesState };

mod create;
mod authorize;
mod delete;

pub fn routes(state: UserRoutesState) -> Router<UserRoutesState> {
    Router::new()
        .route("/", post(create::handler).delete(delete::handler))
        .route("/authorize", post(authorize::handler))
        .with_state(state)
}
