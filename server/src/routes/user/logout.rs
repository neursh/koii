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
    routes::user::UserRoutesState,
};

#[derive(Deserialize)]
pub struct LogoutOptions {
    pub all: Option<bool>,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<UserRoutesState>,
    Query(options): Query<LogoutOptions>
) -> ResponseModel {
    match authorization_info.status {
        AuthorizationStatus::Authorized => {}
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

    let mut token_cache = state.app.cache.token.clone();
    match options.all {
        Some(true) => {
            match token_cache.delete_all(&token.user_id).await {
                Ok(_) => {}
                Err(_) => {
                    return base::response::internal_error(None);
                }
            }
        }
        _ => {
            match token_cache.delete_one(&token).await {
                Ok(_) => {}
                Err(_) => {
                    return base::response::internal_error(None);
                }
            }
        }
    }

    base::response::success(
        StatusCode::OK,
        Some(
            AppendHeaders(
                vec![(
                    SET_COOKIE,
                    "token=; HttpOnly; SameSite=Lax; Secure; Path=/; Domain=.koii.space; Max-Age=0".to_string(),
                )]
            )
        )
    )
}
