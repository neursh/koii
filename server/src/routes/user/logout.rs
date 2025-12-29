use axum::{
    Extension,
    extract::State,
    http::{ StatusCode, header::SET_COOKIE },
    response::AppendHeaders,
};

use crate::{
    base::{ self, response::ResponseModel, session::REFRESH_MAX_AGE },
    cache::refresh::RefreshQuery,
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
    routes::user::UserRoutesState,
};

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<UserRoutesState>
) -> ResponseModel {
    match authorization_info.status {
        AuthorizationStatus::Authorized => {}
        _ => {
            return base::response::error(StatusCode::UNAUTHORIZED, "Get out.", None);
        }
    }

    let refresh = match authorization_info.refresh {
        Some(refresh) => refresh,
        None => {
            return base::response::internal_error(None);
        }
    };

    // Invalidate the refresh token too.
    if
        let Err(error) = state.app.cache.refresh.clone().permit(RefreshQuery {
            user_id: refresh.id,
            created_at: refresh.exp - REFRESH_MAX_AGE,
        }).await
    {
        println!("Logout removing refresh key error: {}", error);
    }

    base::response::success(
        StatusCode::OK,
        Some(
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
        )
    )
}
