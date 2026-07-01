use axum::Router;
use axum::routing::post;

use crate::{ routes::account::AccountRoutesState };

mod create;
mod delete;
mod authorize;

pub fn routes(state: AccountRoutesState) -> Router<AccountRoutesState> {
    Router::new()
        .route("/", post(create::handler).delete(delete::handler))
        .route("/authorize", post(authorize::handler))
        .with_state(state)
}
