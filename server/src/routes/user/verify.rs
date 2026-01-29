use axum::{ Extension, Json, extract::State, http::StatusCode, response::AppendHeaders };
use mongodb::bson;
use nanoid::nanoid;
use serde::Deserialize;

use crate::{
    base::{ self, response::ResponseModel },
    cache::token::TokenQuery,
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
    routes::user::UserRoutesState,
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
        Ok(Some(user_id)) => {
            match
                state.app.cache.token.clone().add(TokenQuery {
                    user_id,
                    created_at: bson::DateTime::now().timestamp_millis(),
                    secret: nanoid!(32),
                }).await
            {
                Ok(header) => {
                    return base::response::success(
                        StatusCode::OK,
                        Some(AppendHeaders(vec![header]))
                    );
                }
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
            tracing::error!(target: "user.verify", "Database failed to verify user: {}", error);
            base::response::internal_error(None)
        }
    };
}
