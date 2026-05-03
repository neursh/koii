use axum::Router;
use axum::routing::get;

use crate::{ routes::account::AccountRoutesState };

mod methods;

pub fn routes(state: AccountRoutesState) -> Router<AccountRoutesState> {
    Router::new().route("/methods", get(methods::handler)).with_state(state)
}
