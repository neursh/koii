use std::time::Duration;

use axum::{ http::HeaderName, response::AppendHeaders };
use cookie_rs::{ Cookie, cookie::SameSite };
use reqwest::header::SET_COOKIE;

use crate::utils::jwt::{ Jwt, TokenClaims, TokenUsage };

pub enum SessionError {
    BadRefreshToken,
    InvalidPrivateKey,
}

/// Will panic if jwt isn't supplied with a valid private key.
pub fn create(
    jwt: &Jwt,
    id: String
) -> Result<AppendHeaders<Vec<(HeaderName, String)>>, SessionError> {
    let created_time = jsonwebtoken::get_current_timestamp();

    let token = if
        let Ok(token) = jwt.generate(TokenClaims {
            usage: TokenUsage::Authorize,
            id: id.clone(),
            exp: created_time + 21600, // 6 hours
        })
    {
        token
    } else {
        return Err(SessionError::InvalidPrivateKey);
    };

    let refresh = if
        let Ok(refresh) = jwt.generate(TokenClaims {
            usage: TokenUsage::Refresh,
            id,
            exp: created_time + 1296000, // 15 days
        })
    {
        refresh
    } else {
        return Err(SessionError::InvalidPrivateKey);
    };

    let token_cookie = construct_cookie("token", token, 21600);
    let refresh_cookie = construct_cookie("refresh", refresh, 1296000);

    Ok(AppendHeaders(vec![(SET_COOKIE, token_cookie), (SET_COOKIE, refresh_cookie)]))
}

pub fn refresh(
    jwt: &Jwt,
    refresh: String
) -> Result<AppendHeaders<Vec<(HeaderName, String)>>, SessionError> {
    if let Ok(refresh_claims) = jwt.verify(refresh) {
        if let TokenUsage::Refresh = refresh_claims.usage {
            return create(jwt, refresh_claims.id);
        }
    }
    Err(SessionError::BadRefreshToken)
}

fn construct_cookie(name: &str, value: String, max_age: u64) -> String {
    Cookie::builder(name, value)
        .domain(".koii.space")
        .path("/")
        .max_age(Duration::from_secs(max_age))
        .same_site(SameSite::Lax)
        .http_only(true)
        .secure(true)
        .build()
        .to_string()
}
