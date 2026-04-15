use axum::{ Extension, extract::State };
use reqwest::StatusCode;

use crate::{
    base::{ self, response::ResponseModel },
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
    routes::user::UserRoutesState,
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

    let totp = match state.app.db.user.document.create_totp(&token.user_id).await {
        Ok(Some(totp)) => { totp }
        Ok(None) => {
            return base::response::error(
                StatusCode::FORBIDDEN,
                "There is an exisiting TOTP. Please delete it first.",
                None
            );
        }
        Err(_) => {
            return base::response::internal_error(None);
        }
    };

    base::response::result(StatusCode::CREATED, totp.url(token.user_id).unwrap().into(), None)
}
