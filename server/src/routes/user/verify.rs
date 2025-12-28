use axum::{ Extension, Json, extract::State, http::StatusCode };
use serde::Deserialize;

use crate::{
    base::{ self, response::ResponseModel },
    routes::user::UserRoutesState,
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
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

    return match state.app.store.users.verify(payload.verify_code).await {
        Ok(Some(id)) => {
            match base::session::create(&state.app.store.refresh, &state.app.jwt, id).await {
                Ok(headers) => base::response::success(StatusCode::OK, Some(headers)),
                Err(_) => base::response::internal_error(None),
            }
        }
        Ok(None) => {
            base::response::error(
                StatusCode::NOT_FOUND,
                "There's no account associated to this verify token.",
                None
            )
        }
        Err(error) => {
            println!("{:?}", error);
            base::response::internal_error(None)
        }
    };
}
