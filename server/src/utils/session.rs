use std::time::Duration;

use axum::{ http::{ HeaderName, header::SET_COOKIE }, response::AppendHeaders };
use cookie_rs::{ Cookie, cookie::SameSite };

use crate::{
    cache::refresh::{ RefreshCache, RefreshCacheQuery },
    consts::{ REFRESH_MAX_AGE, TOKEN_MAX_AGE },
    utils::jwt::{ Jwt, TokenClaims, TokenUsage },
};

pub enum SessionError {
    DatabaseError,
    BadRefreshToken,
}

/// Will panic if jwt isn't supplied with a valid private key.
pub async fn create(
    refresh_cache: &mut RefreshCache,
    jwt: &Jwt,
    id: String
) -> Result<AppendHeaders<Vec<(HeaderName, String)>>, SessionError> {
    let created_at = jsonwebtoken::get_current_timestamp() as i64;

    let token = jwt.generate(TokenClaims {
        usage: TokenUsage::Authorize,
        id: id.clone(),
        exp: created_at + TOKEN_MAX_AGE,
    });

    let refresh = jwt.generate(TokenClaims {
        usage: TokenUsage::Refresh,
        id: id.clone(),
        exp: created_at + REFRESH_MAX_AGE,
    });

    let token_cookie = construct_cookie("token", token, TOKEN_MAX_AGE);
    let refresh_cookie = construct_cookie("refresh", refresh, REFRESH_MAX_AGE);

    return match
        refresh_cache.add(RefreshCacheQuery {
            user_id: id,
            created_at: created_at,
        }).await
    {
        Ok(_) => Ok(AppendHeaders(vec![(SET_COOKIE, token_cookie), (SET_COOKIE, refresh_cookie)])),
        Err(_) => Err(SessionError::DatabaseError),
    };
}

/// Will panic if jwt isn't supplied with a valid private key.
pub async fn refresh_from_claims(
    refresh_cache: &mut RefreshCache,
    jwt: &Jwt,
    refresh: TokenClaims
) -> Result<AppendHeaders<Vec<(HeaderName, String)>>, SessionError> {
    if let TokenUsage::Refresh = refresh.usage {
        return create(refresh_cache, jwt, refresh.id).await;
    }
    Err(SessionError::BadRefreshToken)
}

/// Will panic if jwt isn't supplied with a valid private key.
pub async fn refresh(
    refresh_cache: &mut RefreshCache,
    jwt: &Jwt,
    refresh: String
) -> Result<AppendHeaders<Vec<(HeaderName, String)>>, SessionError> {
    if let Some(refresh_claims) = jwt.verify(refresh) {
        if let TokenUsage::Refresh = refresh_claims.usage {
            return create(refresh_cache, jwt, refresh_claims.id).await;
        }
    }
    Err(SessionError::BadRefreshToken)
}

fn construct_cookie(name: &str, value: String, max_age: i64) -> String {
    Cookie::builder(name, value)
        .domain(".koii.space")
        .path("/")
        .max_age(Duration::from_secs(max_age as u64))
        .same_site(SameSite::Lax)
        .http_only(true)
        .secure(true)
        .build()
        .to_string()
}
