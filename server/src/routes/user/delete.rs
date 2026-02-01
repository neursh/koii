use axum::{
    Extension,
    extract::State,
    http::{ StatusCode, header::SET_COOKIE },
    response::AppendHeaders,
};

use crate::{
    base::{ self, response::ResponseModel },
    routes::user::UserRoutesState,
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
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

    // Safely remove the user first, if fail, don't remove token.
    match state.app.db.user.store.entry.delete(&token.user_id).await {
        Ok(_) => {}
        Err(error) => {
            tracing::error!(target: "endpoint.delete.profile", "{}\n{}", token.user_id, error);
            return base::response::internal_error(None);
        }
    }

    // User now gone, delete token in cache.
    return match state.app.db.user.cache.token.clone().delete_all(&token.user_id).await {
        Ok(_) => {
            base::response::success(
                StatusCode::OK,
                Some(
                    AppendHeaders(
                        vec![(
                            SET_COOKIE,
                            "token=; HttpOnly; SameSite=Lax; Secure; Path=/; Domain=.koii.space; Max-Age=0".to_string(),
                        )]
                    )
                )
            )
        }
        Err(error) => {
            tracing::error!(target: "endpoint.delete.token", "{}\n{}", token.user_id, error);
            base::response::internal_error(None)
        }
    };
}
