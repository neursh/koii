use axum::{ Extension, extract::State };

use crate::{
    base::response::ResponseModel,
    routes::user::UserRoutesState,
    utils::cookie_query::AuthorizationInfo,
};

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<UserRoutesState>
) -> ResponseModel {}
