use std::sync::Arc;

use axum::{
    extract::{ Request, State },
    http::HeaderMap,
    middleware::Next,
    response::IntoResponse,
};
use cookie_rs::Cookie;

use crate::{ AppState, utils::jwt::{ Jwt, TokenClaims, TokenUsage } };

#[derive(Clone)]
pub enum AuthorizationStatus {
    Authorized,
    Unauthorized,
    /// Refresh failed also means the user is unauthorized.
    RefreshFailed,
}

#[derive(Clone)]
pub struct AuthorizationInfo {
    pub token: Option<TokenClaims>,
    pub refresh: Option<TokenClaims>,
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
        let info = parse_cookies(&state.jwt, value_str).await;

        request.extensions_mut().insert(info);
    } else {
        request.extensions_mut().insert(AuthorizationInfo {
            token: None,
            refresh: None,
            status: AuthorizationStatus::Unauthorized,
        });
    }

    next.run(request).await
}

async fn parse_cookies(jwt: &Jwt, cookies_str: &str) -> AuthorizationInfo {
    let mut token = None;
    let mut refresh = None;

    for cookie in cookies_str.split("; ") {
        if let Ok(payload) = Cookie::parse(cookie) {
            if
                payload.name() == "token" &&
                let Ok(claims) = jwt.verify(payload.value().to_string()) &&
                let TokenUsage::Authorize = claims.usage
            {
                token = Some(claims);
            }
            if
                payload.name() == "refresh" &&
                let Ok(claims) = jwt.verify(payload.value().to_string()) &&
                let TokenUsage::Refresh = claims.usage
            {
                refresh = Some(claims);
            }
        }
    }

    let status = if token.is_some() && refresh.is_some() {
        AuthorizationStatus::Authorized
    } else {
        AuthorizationStatus::Unauthorized
    };

    AuthorizationInfo {
        token,
        refresh,
        status,
    }
}
