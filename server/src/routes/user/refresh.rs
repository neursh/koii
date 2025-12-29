use axum::{ Extension, extract::State, http::StatusCode };
use mongodb::bson;

use crate::{
    base::{ self, response::ResponseModel, session::{ REFRESH_MAX_AGE, SessionError } },
    cache::refresh::RefreshQuery,
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
    routes::user::UserRoutesState,
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

    let refresh = match authorization_info.refresh {
        Some(refresh) => refresh,
        None => {
            return base::response::error(StatusCode::UNAUTHORIZED, "Get out.", None);
        }
    };
    let refresh_creation = refresh.exp - REFRESH_MAX_AGE;

    // First gate: Check with the user info to make sure that the token is not invalidated.
    match
        state.app.store.users.check_accept_refresh(
            &refresh.id,
            bson::DateTime::from_millis(refresh_creation * 1000)
        ).await
    {
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
    match
        state.app.cache.refresh.clone().permit(RefreshQuery {
            user_id: refresh.id.clone(),
            created_at: refresh_creation,
        }).await
    {
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
            &mut state.app.cache.refresh.clone(),
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
