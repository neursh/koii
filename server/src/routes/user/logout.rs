use axum::{
    Extension,
    extract::State,
    http::{ StatusCode, header::SET_COOKIE },
    response::AppendHeaders,
};
use mongodb::bson;

use crate::{
    base::{ self, response::ResponseModel, session::REFRESH_MAX_AGE },
    routes::user::UserRoutesState,
    utils::cookie_query::{ AuthorizationInfo, AuthorizationStatus },
};

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<UserRoutesState>
) -> ResponseModel {
    match authorization_info.status {
        AuthorizationStatus::Authorized => {},
        _ => {
            return base::response::error(StatusCode::UNAUTHORIZED, "Get out.", None);
        }
    }

    let refresh = authorization_info.refresh.unwrap();

    // Invalidate the refresh token too.
    if
        let Err(error) = state.app.database.refresh.permit(
            &refresh.id,
            bson::DateTime::from_millis((refresh.exp - REFRESH_MAX_AGE) * 1000)
        ).await
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
