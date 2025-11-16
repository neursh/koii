use axum::{
    extract::{ Request, State },
    http::{ HeaderMap, HeaderName, HeaderValue },
    middleware::Next,
    response::IntoResponse,
};
use cookie_rs::Cookie;
use mongodb::bson;

use crate::{
    base::{ self, session::{ REFRESH_MAX_AGE, TOKEN_MAX_AGE } },
    database::refresh::RefreshStore,
    utils::jwt::{ Jwt, TokenClaims, TokenUsage },
};

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
    pub status: AuthorizationStatus,
}

#[derive(Clone)]
pub struct AuthorizationState {
    pub jwt: Jwt,
    pub refresh_store: RefreshStore,
}

pub async fn authorize(
    State(state): State<AuthorizationState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next
) -> impl IntoResponse {
    let raw_cookies = headers.get("cookie");
    let mut refreshed_tokens = None;

    if let Some(value) = raw_cookies && let Ok(value_str) = value.to_str() {
        let (refreshed, info) = parse_cookies(&state.refresh_store, &state.jwt, value_str).await;

        refreshed_tokens = refreshed;
        request.extensions_mut().insert(info);
    } else {
        request.extensions_mut().insert(AuthorizationInfo {
            token: None,
            status: AuthorizationStatus::Unauthorized,
        });
    }

    let mut response = next.run(request).await;

    if let Some(refreshed_tokens) = refreshed_tokens {
        for item in refreshed_tokens {
            response.headers_mut().append(item.0, HeaderValue::from_str(&item.1).unwrap());
        }
    }

    response
}

async fn parse_cookies(
    refresh_store: &RefreshStore,
    jwt: &Jwt,
    cookies_str: &str
) -> (Option<Vec<(HeaderName, String)>>, AuthorizationInfo) {
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

    let mut status = if token.is_some() {
        AuthorizationStatus::Authorized
    } else {
        AuthorizationStatus::Unauthorized
    };

    let mut headers = None;

    // Recover authorization status when there is a refresh token.
    if let Some(refresh) = refresh && let AuthorizationStatus::Unauthorized = status {
        let permit = match
            refresh_store.permit(
                &refresh.id,
                bson::DateTime::from_millis((refresh.exp - REFRESH_MAX_AGE) * 1000)
            ).await
        {
            Ok(permit) => permit,
            Err(_) => {
                status = AuthorizationStatus::RefreshFailed;
                false
            }
        };

        if permit {
            match base::session::refresh_from_claims(refresh_store, jwt, refresh.clone()).await {
                Ok(cookies) => {
                    headers = Some(cookies.0);
                    token = Some(TokenClaims {
                        usage: TokenUsage::Authorize,
                        id: refresh.id,
                        exp: (jsonwebtoken::get_current_timestamp() as i64) + TOKEN_MAX_AGE,
                    });
                    status = AuthorizationStatus::Authorized;
                }
                Err(_) => {
                    status = AuthorizationStatus::RefreshFailed;
                }
            };
        }
    }

    (
        headers,
        AuthorizationInfo {
            token,
            status,
        },
    )
}
