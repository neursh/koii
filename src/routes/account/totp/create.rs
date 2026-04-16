use axum::{ Extension, Json, extract::State };
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    base::{ self, response::ResponseModel },
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
    routes::account::AccountRoutesState,
    utils::totp::Totp,
};

#[derive(Deserialize)]
pub struct CreatePayload {
    name: String,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<AccountRoutesState>,
    Json(payload): Json<CreatePayload>
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

    let totp = match Totp::new(payload.name) {
        Ok(totp) => totp,
        Err(_) => {
            return base::response::internal_error(None);
        }
    };

    match state.app.db.account.document.add_totp(&token.account_id, &totp).await {
        Ok(true) => {} // TOTP added, passing down.
        Ok(false) => {
            return base::response::error(
                StatusCode::FORBIDDEN,
                "There is an exisiting TOTP. Please delete it first.",
                None
            );
        }
        Err(_) => {
            return base::response::internal_error(None);
        }
    }

    base::response::result(StatusCode::CREATED, totp.url.into(), None)
}
