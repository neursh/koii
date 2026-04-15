use std::sync::Arc;

use axum::{
    extract::{ Request, State },
    http::HeaderMap,
    middleware::Next,
    response::IntoResponse,
};
use cookie_rs::Cookie;
use reqwest::header::COOKIE;

use crate::{ AppState, utils::jwt::TokenClaims };

#[derive(Clone)]
pub enum AuthorizationStatus {
    Authorized,
    Unauthorized,
}

#[derive(Clone)]
pub struct AuthorizationInfo {
    pub token: Option<TokenClaims>,
    pub status: AuthorizationStatus,
}

pub async fn authorize(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next
) -> impl IntoResponse {
    if let Some(cookies) = headers.get(COOKIE) && let Ok(cookies) = cookies.to_str() {
        request.extensions_mut().insert(parse_cookies(state, cookies).await);
    } else {
        request.extensions_mut().insert(AuthorizationInfo {
            token: None,
            status: AuthorizationStatus::Unauthorized,
        });
    }

    next.run(request).await
}

async fn parse_cookies(state: Arc<AppState>, cookies: &str) -> AuthorizationInfo {
    let mut token = None;

    for cookie in cookies.split("; ") {
        if let Ok(payload) = Cookie::parse(cookie) && payload.name() == "token" {
            token = state.jwt.verify(payload.value());
            break;
        }
    }

    return match token {
        Some(token) => {
            match state.db.user.token.clone().authorize(&token).await {
                Ok(true) =>
                    AuthorizationInfo {
                        token: Some(token),
                        status: AuthorizationStatus::Authorized,
                    },
                _ =>
                    AuthorizationInfo {
                        token: None,
                        status: AuthorizationStatus::Unauthorized,
                    },
            }
        }
        None =>
            AuthorizationInfo {
                token: None,
                status: AuthorizationStatus::Unauthorized,
            },
    };
}
