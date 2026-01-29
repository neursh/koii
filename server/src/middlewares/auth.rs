use std::sync::Arc;

use axum::{
    extract::{ Request, State },
    http::HeaderMap,
    middleware::Next,
    response::IntoResponse,
};
use cookie_rs::Cookie;

use crate::{ AppState, cache::token::TokenQuery };

#[derive(Clone)]
pub enum AuthorizationStatus {
    Authorized,
    Unauthorized,
}

#[derive(Clone)]
pub struct AuthorizationInfo {
    pub token: Option<TokenQuery>,
    pub status: AuthorizationStatus,
}

pub async fn authorize(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next
) -> impl IntoResponse {
    let raw_cookies = headers.get("cookie");

    if let Some(value) = raw_cookies && let Ok(value_str) = value.to_str() {
        request.extensions_mut().insert(parse_cookies(state, value_str).await);
    } else {
        request.extensions_mut().insert(AuthorizationInfo {
            token: None,
            status: AuthorizationStatus::Unauthorized,
        });
    }

    next.run(request).await
}

async fn parse_cookies(state: Arc<AppState>, cookies_str: &str) -> AuthorizationInfo {
    let mut token = None;

    for cookie in cookies_str.split("; ") {
        if let Ok(payload) = Cookie::parse(cookie) && payload.name() == "token" {
            let raw_token: Vec<&str> = payload.value().split(".").collect();
            token = parse_token(raw_token);
            break;
        }
    }

    return match token {
        Some(token) => {
            match state.cache.token.clone().authorize(&token).await {
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

fn parse_token(raw_token: Vec<&str>) -> Option<TokenQuery> {
    let created_at = match raw_token.get(1)?.to_owned().parse::<i64>() {
        Ok(created_at) => created_at,
        Err(_) => {
            return None;
        }
    };

    Some(TokenQuery {
        user_id: raw_token.get(0)?.to_owned().to_owned(),
        created_at,
        secret: raw_token.get(2)?.to_owned().to_owned(),
    })
}
