use axum::{Extension, extract::State};

use crate::{middlewares::auth::AuthorizationInfo, routes::user::UserRoutesState};

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<UserRoutesState>
) {
    
}
