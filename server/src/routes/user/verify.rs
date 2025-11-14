use axum::{ Json, extract::State };
use reqwest::StatusCode;
use serde_json::json;
use serde::Deserialize;

use crate::{
    base::{ self, response::ResponseModel },
    routes::user::RouteState,
    utils::jwt::UserClaims,
};

#[derive(Deserialize)]
pub struct VerifyPayload {
    pub verify_code: String,
}

pub async fn handler(
    State(state): State<RouteState>,
    Json(payload): Json<VerifyPayload>
) -> (StatusCode, Json<ResponseModel>) {
    match state.koii_database.users.verify(payload.verify_code).await {
        Ok(done) => {
            if let Some(_id) = done {
                let token = state.jwt.generate(UserClaims {
                    _id,
                    exp: jsonwebtoken::get_current_timestamp() + 60 * 15,
                });

                return match token {
                    Ok(token) => base::response::result(StatusCode::OK, json!({ "token": token })),
                    Err(_) => base::response::internal_error(),
                };
            } else {
                return base::response::error(
                    StatusCode::NOT_FOUND,
                    "There's no account associated to this verify token."
                );
            }
        }
        Err(s) => {
            println!("{:?}", s);
            return base::response::internal_error();
        }
    }
}
