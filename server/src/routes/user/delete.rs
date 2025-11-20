use axum::{
    Extension,
    extract::State,
    http::{ HeaderName, StatusCode, header::SET_COOKIE },
    response::AppendHeaders,
};

use crate::{
    base::{ self, response::ResponseModel },
    routes::user::UserRoutesState,
    utils::cookie_query::{ AuthorizationInfo, AuthorizationStatus },
};

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<UserRoutesState>
) -> ResponseModel {
    match authorization_info.status {
        AuthorizationStatus::Authorized => (),
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

    match state.app.database.users.delete(token.id).await {
        Ok(true) => {
            return base::response::success(StatusCode::OK, Some(clear_tokens_header()));
        }
        Ok(false) => {
            return base::response::error(
                StatusCode::CONFLICT,
                "The user is already deleted. Why is the cookie still here?",
                Some(clear_tokens_header())
            );
        }
        Err(error) => {
            eprintln!("{}", error);
            return base::response::internal_error(None);
        }
    };
}

fn clear_tokens_header() -> AppendHeaders<Vec<(HeaderName, String)>> {
    AppendHeaders(
        vec![
            (
                SET_COOKIE,
                "token=; HttpOnly; SameSite=Lax; Secure; Path=/; Domain=.koii.space; Max-Age=0".to_string(),
            ),
            (
                SET_COOKIE,
                "refresh=; HttpOnly; SameSite=Lax; Secure; Path=/; Domain=.koii.space; Max-Age=0".to_string(),
            )
        ]
    )
}
