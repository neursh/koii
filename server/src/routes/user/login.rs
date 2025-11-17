use axum::{ Extension, Json, extract::State, http::StatusCode };
use mongodb::bson;
use serde::Deserialize;

use crate::{
    base::{ self, response::ResponseModel },
    routes::user::RouteState,
    services::verify_pass::VerifyPassRequest,
    utils::{ checks::credentials_checks, middlewares::{ AuthorizationInfo, AuthorizationStatus } },
};

#[derive(Deserialize, Clone)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<RouteState>,
    Json(payload): Json<LoginPayload>
) -> ResponseModel {
    if let AuthorizationStatus::Authorized = authorization_info.status {
        return base::response::error(
            StatusCode::FORBIDDEN,
            "There's already an active user.",
            None
        );
    }

    let payload_task = payload.clone();
    match
        tokio::task::spawn_blocking(move ||
            credentials_checks(&payload_task.email, &payload_task.password)
        ).await
    {
        Ok(result) => {
            if let Err(bad) = result {
                return bad;
            }
        }
        _ => {
            return base::response::internal_error(None);
        }
    }

    let edge_user = state.database.users.get_one(
        bson::doc! {
            "email": payload.email
        }
    ).await;
    let user = match edge_user {
        Ok(user) => {
            match user {
                Some(user) => user,
                None => {
                    return base::response::error(
                        StatusCode::FORBIDDEN,
                        "Wrong email or password.",
                        None
                    );
                }
            }
        }
        Err(error) => {
            eprintln!("Database error: {:?}", error.kind);
            return base::response::internal_error(None);
        }
    };

    let result = match
        state.services.verify_pass.send(VerifyPassRequest {
            password: payload.password,
            hash: user.password_hash,
        }).await
    {
        Ok(Some(result)) => result,
        _ => {
            return base::response::internal_error(None);
        }
    };

    if result {
        return match base::session::create(&state.database.refresh, &state.jwt, user._id).await {
            Ok(headers) => base::response::success(StatusCode::OK, Some(headers)),
            Err(_) => base::response::internal_error(None),
        };
    }

    base::response::error(StatusCode::FORBIDDEN, "Wrong email or password.", None)
}
