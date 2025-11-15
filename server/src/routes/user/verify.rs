use axum::{ Json, extract::State };
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{ base::{ self, response::ResponseModel }, routes::user::RouteState };

#[derive(Deserialize)]
pub struct VerifyPayload {
    pub verify_code: String,
}

pub async fn handler(
    State(state): State<RouteState>,
    Json(payload): Json<VerifyPayload>
) -> ResponseModel {
    match state.koii_database.users.verify(payload.verify_code).await {
        Ok(done) => {
            if let Some(id) = done {
                return match base::session::create(&state.jwt, id) {
                    Ok(session_cookie) =>
                        base::response::success(StatusCode::OK, Some(session_cookie)),
                    Err(_) => base::response::internal_error(None),
                };
            } else {
                return base::response::error(
                    StatusCode::NOT_FOUND,
                    "There's no account associated to this verify token.",
                    None
                );
            }
        }
        Err(s) => {
            println!("{:?}", s);
            return base::response::internal_error(None);
        }
    }
}
