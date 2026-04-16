use axum::{ Extension, Json, extract::State, http::StatusCode };
use serde::Deserialize;

use crate::{
    base::{ self, response::ResponseModel },
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
    routes::account::AccountRoutesState,
};

#[derive(Deserialize)]
pub struct AuthorizePayload {
    totp_code: String,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<AccountRoutesState>,
    Json(payload): Json<AuthorizePayload>
) -> ResponseModel {
    match authorization_info.status {
        AuthorizationStatus::Authorized => {} // Authorized, passing down.
        _ => {
            return base::response::error(StatusCode::UNAUTHORIZED, "Get out.", None);
        }
    }

    let token = match authorization_info.token {
        Some(token) => token,
        None => {
            return base::response::internal_error(None);
        }
    };

    let totp = match state.app.db.account.document.get_totp(&token.account_id).await {
        Ok(Some(totp)) => totp,
        Ok(None) => {
            return base::response::error(
                StatusCode::NOT_FOUND,
                "The account doesn't enable TOTP.",
                None
            );
        }
        Err(_) => {
            return base::response::internal_error(None);
        }
    };

    match totp.verify(&payload.totp_code) {
        Ok(_) => todo!(),
        Err(_) => todo!(),
    }
}
