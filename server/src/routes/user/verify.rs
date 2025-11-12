use axum::{ Json, extract::State };
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{ base::{ self, response::ResponseModel }, routes::user::RouteState };

#[derive(Deserialize)]
pub struct VerifyPayload {
    pub token: String,
}

pub async fn handler(
    State(state): State<RouteState>,
    Json(payload): Json<VerifyPayload>
) -> (StatusCode, Json<ResponseModel>) {
    match state.koii_database.users.verify(payload.token).await {
        Ok(done) => {
            if done {
                return base::response::success(StatusCode::OK);
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
