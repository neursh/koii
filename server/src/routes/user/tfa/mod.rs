use axum::Router;

use crate::{ routes::user::UserRoutesState };

mod totp;
mod passkey;

pub fn routes(state: UserRoutesState) -> Router<UserRoutesState> {
    Router::new()
        .nest("/totp", totp::routes(state.clone()))
        .nest("/passkey", passkey::routes(state.clone()))
        .with_state(state)
}
