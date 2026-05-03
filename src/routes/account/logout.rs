use axum::{
    Extension,
    extract::{ Query, State },
    http::{ StatusCode, header::SET_COOKIE },
    response::AppendHeaders,
};
use serde::Deserialize;

use crate::{
    base::{ self, response::ResponseModel },
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
    routes::account::AccountRoutesState,
    utils::cookies,
};

#[derive(Deserialize)]
pub struct LogoutOptions {
    pub all: Option<bool>,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<AccountRoutesState>,
    Query(options): Query<LogoutOptions>
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

    let mut token_cache = state.app.db.token.clone();
    match options.all {
        Some(true) => {
            match token_cache.revoke_all(&token.account_id).await {
                Ok(_) => {} // Revoked, passing down.
                Err(error) => {
                    tracing::error!(
                        "Unable to revoke all tokens for {}: {}",
                        &token.account_id,
                        error
                    );
                    return base::response::internal_error(None);
                }
            }
        }
        _ => {
            match token_cache.revoke(&token).await {
                Ok(_) => {} // Revoked, passing down.
                Err(error) => {
                    tracing::error!(
                        "Unable to revoke {} token for {}: {}",
                        &token.identifier,
                        &token.account_id,
                        error
                    );
                    return base::response::internal_error(None);
                }
            }
        }
    }

    base::response::success(
        StatusCode::OK,
        Some(
            AppendHeaders(
                vec![
                    (SET_COOKIE, cookies::remove("token", "/")),
                    (SET_COOKIE, cookies::remove("refresh", "/account/refresh_token"))
                ]
            )
        )
    )
}
