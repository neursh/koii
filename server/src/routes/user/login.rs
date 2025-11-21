use axum::{ Extension, Json, extract::State, http::StatusCode };
use mongodb::bson;
use serde::Deserialize;

use crate::{
    base::{ self, response::ResponseModel },
    routes::user::UserRoutesState,
    services::verify_pass::VerifyPassRequest,
    utils::{ checks, cookie_query::{ AuthorizationInfo, AuthorizationStatus } },
};

#[derive(Deserialize, Clone)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<UserRoutesState>,
    Json(payload): Json<LoginPayload>
) -> ResponseModel {
    if let AuthorizationStatus::Authorized = authorization_info.status {
        return base::response::error(
            StatusCode::FORBIDDEN,
            "There's already an active user.",
            None
        );
    }

    match checks::credentials(&payload.email, &payload.password) {
        Ok(_) => {} // valid, move on
        Err(error) => {
            return error;
        }
    }

    let edge_user = state.app.database.users.get_one(
        bson::doc! {
            "email": payload.email
        }
    ).await;
    let user = match edge_user {
        Ok(Some(user)) => user,
        Ok(None) => {
            return base::response::error(StatusCode::FORBIDDEN, "Wrong email or password.", None);
        }
        Err(error) => {
            eprintln!("Database error: {:?}", error.kind);
            return base::response::internal_error(None);
        }
    };

    match
        state.app.services.verify_pass.send(VerifyPassRequest {
            password: payload.password,
            hash: user.password_hash,
        }).await
    {
        Ok(Some(true)) => {
            match base::session::create(&state.app.database.refresh, &state.app.jwt, user.id).await {
                Ok(headers) => {
                    return base::response::success(StatusCode::OK, Some(headers));
                }
                Err(_) => {
                    return base::response::internal_error(None);
                }
            };
        }
        Ok(Some(false)) => {
            return base::response::error(StatusCode::FORBIDDEN, "Wrong email or password.", None);
        }
        // Err & Ok(None)
        _ => {
            return base::response::internal_error(None);
        }
    };
}
