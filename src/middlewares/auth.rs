use std::sync::Arc;

use axum::{
    extract::{ Request, State },
    http::HeaderMap,
    middleware::Next,
    response::IntoResponse,
};
use cookie_rs::CookieJar;
use reqwest::header::COOKIE;

use crate::{ AppState, utils::jwt::{ TokenClaims, TokenKind } };

#[derive(Clone)]
pub struct AuthorizationInfo {
    /// This value satisfies when `token` or `refresh` valid.
    pub active: bool,
    pub token: Option<TokenClaims>,
    pub refresh: Option<TokenClaims>,
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
            active: false,
            token: None,
            refresh: None,
        });
    }

    next.run(request).await
}

async fn parse_cookies(state: Arc<AppState>, cookies: &str) -> AuthorizationInfo {
    let mut token = None;
    let mut refresh = None;
    let mut active = false;

    let Ok(jar) = CookieJar::parse(cookies) else {
        return AuthorizationInfo {
            active: false,
            token: None,
            refresh: None,
        };
    };

    if let Some(payload) = jar.get("token") {
        token = state.jwt.verify(payload.value(), TokenKind::AUTHENTICATION);
        active = token.is_some();
    }

    if let Some(payload) = jar.get("refresh") {
        refresh = state.jwt.verify(payload.value(), TokenKind::REFRESH);
        active = refresh.is_some();
    }

    AuthorizationInfo {
        active,
        token,
        refresh,
    }
}
