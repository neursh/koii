use axum::{ Extension, extract::State, http::StatusCode };
use mongodb::bson;

use crate::{
    base::{ self, response::ResponseModel, session::{ REFRESH_MAX_AGE, SessionError } },
    routes::user::UserRoutesState,
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
};

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<UserRoutesState>
) -> ResponseModel {
    match authorization_info.status {
        AuthorizationStatus::Authorized => {
            return base::response::error(
                StatusCode::CONFLICT,
                "The user token is still active.",
                None
            );
        }
        AuthorizationStatus::Unauthorized => {
            return base::response::error(StatusCode::UNAUTHORIZED, "Get out.", None);
        }
        AuthorizationStatus::RefreshActive => {}
    }
    let refresh = authorization_info.refresh.unwrap();
    let refresh_creation = bson::DateTime::from_millis((refresh.exp - REFRESH_MAX_AGE) * 1000);

    // First gate: Check with the user info to make sure that the token is not invalidated.
    match state.app.database.users.check_accept_refresh(&refresh.id, refresh_creation).await {
        Ok(true) => {} // valid, move on
        Ok(false) => {
            return base::response::error(StatusCode::UNAUTHORIZED, "Get out.", None);
        }
        Err(_) => {
            return base::response::internal_error(None);
        }
    }

    // Second gate: Check with the refresh base to make sure that the token is a valid issued refresh token,
    // and could only be used once.
    match state.app.database.refresh.permit(&refresh.id, refresh_creation).await {
        Ok(true) => {} // valid, move on
        Ok(false) => {
            return base::response::error(StatusCode::UNAUTHORIZED, "Get out.", None);
        }
        Err(_) => {
            return base::response::internal_error(None);
        }
    }

    match
        base::session::refresh_from_claims(
            &state.app.database.refresh,
            &state.app.jwt,
            refresh
        ).await
    {
        Ok(headers) => {
            return base::response::success(StatusCode::OK, Some(headers));
        }
        Err(SessionError::BadRefreshToken) => {
            return base::response::error(StatusCode::UNAUTHORIZED, "Get out.", None);
        }
        Err(SessionError::DatabaseError) => {
            return base::response::internal_error(None);
        }
    }
}
