use axum::{ Extension, Json, extract::State, http::StatusCode };
use serde::Deserialize;

use crate::{
    base::{ self, response::ResponseModel },
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
    routes::user::UserRoutesState,
    utils,
};

#[derive(Deserialize)]
pub struct VerifyPayload {
    pub verify_code: String,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<UserRoutesState>,
    Json(payload): Json<VerifyPayload>
) -> ResponseModel {
    if let AuthorizationStatus::Authorized = authorization_info.status {
        return base::response::error(
            StatusCode::FORBIDDEN,
            "There's already an active user.",
            None
        );
    }

    // Get user id from cache.
    let user_id = match state.app.cache.verify.clone().permit(&payload.verify_code).await {
        Ok(Some(user_id)) => user_id,
        Ok(None) => {
            return base::response::error(
                StatusCode::NOT_FOUND,
                "There's no account associated to this verify token.",
                None
            );
        }
        Err(error) => {
            println!("{:?}", error);
            return base::response::internal_error(None);
        }
    };

    // Confirm with database.
    match state.app.store.users.confirm(&user_id).await {
        Ok(true) => {}
        Ok(false) => {
            return base::response::error(
                StatusCode::NOT_FOUND,
                "There's no account associated to this verify token.",
                None
            );
        }
        Err(error) => {
            println!("{:?}", error);
            return base::response::internal_error(None);
        }
    }

    // Return session.
    match
        utils::session::create(&mut state.app.cache.refresh.clone(), &state.app.jwt, user_id).await
    {
        Ok(headers) => {
            return base::response::success(StatusCode::OK, Some(headers));
        }
        Err(_) => {
            return base::response::internal_error(None);
        }
    };
}
