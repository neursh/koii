use axum::{ Extension, Json, extract::State, http::StatusCode };
use mongodb::bson;
use serde::Deserialize;
use validator::Validate;

use crate::{
    base::{ self, response::ResponseModel },
    routes::user::UserRoutesState,
    worker::verify_pass::VerifyPassRequest,
    utils::{ cookie_query::{ AuthorizationInfo, AuthorizationStatus } },
};

#[derive(Deserialize, Validate, Clone)]
pub struct LoginPayload {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 12))]
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

    // Validate before processing.
    match payload.validate() {
        Ok(_) => {} // valid, move on
        Err(error) => {
            let (bad_field, _) = error.errors().iter().next().unwrap();
            if bad_field == "email" {
                return base::response::error(StatusCode::BAD_REQUEST, "Invalid email.", None);
            }
            if bad_field == "password" {
                return base::response::error(
                    StatusCode::BAD_REQUEST,
                    "Password must be longer than 12 characters.",
                    None
                );
            }

            return base::response::internal_error(None);
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
        state.app.worker.verify_pass.send(VerifyPassRequest {
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
