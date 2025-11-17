use axum::{
    Extension,
    extract::State,
    http::{ StatusCode, header::SET_COOKIE },
    response::AppendHeaders,
};

use crate::{
    base::{ self, response::ResponseModel },
    routes::user::RouteState,
    utils::middlewares::{ AuthorizationInfo, AuthorizationStatus },
};

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<RouteState>
) -> ResponseModel {
    match authorization_info.status {
        AuthorizationStatus::Authorized => (),
        _ => {
            return base::response::error(StatusCode::UNAUTHORIZED, "Get out.", None);
        }
    }

    let token = authorization_info.token.unwrap();

    let result = match state.database.users.delete(token.id).await {
        Ok(result) => result,
        Err(error) => {
            eprintln!("{}", error);
            return base::response::internal_error(None);
        }
    };

    if result {
        return base::response::success(
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
        );
    }

    base::response::error(
        StatusCode::CONFLICT,
        "The user is already deleted. Why is the cookie still here?",
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
