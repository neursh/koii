use axum::Router;
use axum::routing::post;

use crate::{ routes::account::AccountRoutesState };

mod create;
mod challange;
mod authorize;
mod delete;

pub fn routes(state: AccountRoutesState) -> Router<AccountRoutesState> {
    Router::new()
        .route("/", post(create::handler).delete(delete::handler))
        .route("/challange", post(challange::handler))
        .route("/authorize", post(authorize::handler))
        .with_state(state)
}
