use std::fmt::Display;

use axum::{ http::{ HeaderName, header::SET_COOKIE }, response::AppendHeaders };

use crate::{
    consts::{ REFRESH_MAX_AGE, TOKEN_MAX_AGE },
    utils::{ cookies, jwt::{ Jwt, TokenClaims, TokenUsage } },
};

pub enum SessionError {
    DatabaseError,
    BadRefreshToken,
}
impl Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
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

    let token_cookie = cookies::construct("token", token, TOKEN_MAX_AGE);
    let refresh_cookie = cookies::construct("refresh", refresh, REFRESH_MAX_AGE);

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
