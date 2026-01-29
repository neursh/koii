use axum::{ Extension, Json, extract::State, http::StatusCode, response::AppendHeaders };
use mongodb::bson;
use nanoid::nanoid;
use serde::Deserialize;
use validator::Validate;

use crate::{
    base::{ self, response::ResponseModel },
    cache::token::TokenQuery,
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
    routes::user::UserRoutesState,
    workers::verify_pass::VerifyPassRequest,
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

    match payload.validate() {
        Ok(_) => {}
        Err(field) => {
            if let Some(field) = field.errors().iter().next() {
                return base::response::error(
                    StatusCode::BAD_REQUEST,
                    &format!("A field is not satisfied: {}", field.0),
                    None
                );
            }
            return base::response::internal_error(None);
        }
    }

    let edge_user = state.app.store.users.get_one(
        bson::doc! {
            "email": &payload.email
        }
    ).await;
    let user = match edge_user {
        Ok(Some(user)) => user,
        Ok(None) => {
            return base::response::error(StatusCode::FORBIDDEN, "Wrong email or password.", None);
        }
        Err(error) => {
            tracing::error!(target: "user.login", "{}\n{}", payload.email, error);
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
            match
                state.app.cache.token.clone().add(TokenQuery {
                    user_id: user.id,
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
                Err(error) => {
                    tracing::error!(target: "user.login", "{}\n{}", payload.email, error);
                    return base::response::internal_error(None);
                }
            };
        }
        Ok(Some(false)) => {
            return base::response::error(StatusCode::FORBIDDEN, "Wrong email or password.", None);
        }
        _ => {
            tracing::error!(target: "user.login", "Verify password worker failure.");
            return base::response::internal_error(None);
        }
    };
}
