use axum::{
    Extension,
    Json,
    extract::State,
    http::{ StatusCode, header::SET_COOKIE },
    response::AppendHeaders,
};
use mongodb::bson;
use serde::Deserialize;
use validator::Validate;

use crate::{
    base::{ self, response::ResponseModel },
    consts::{ REFRESH_MAX_AGE, TOKEN_MAX_AGE },
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
    routes::user::UserRoutesState,
    utils::cookies,
    workers::verify_pass::VerifyPassRequest,
};

#[derive(Deserialize, Validate, Clone)]
pub struct LoginPayload {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 12))]
    pub password: String,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<UserRoutesState>,
    Json(payload): Json<LoginPayload>
) -> ResponseModel {
    if let AuthorizationStatus::Authorized = authorization_info.status {
        return base::response::error(
            StatusCode::FORBIDDEN,
            "There's already an active user.",
            None
        );
    }

    match payload.validate() {
        Ok(_) => {}
        Err(field) => {
            if let Some(field) = field.errors().iter().next() {
                return base::response::error(
                    StatusCode::BAD_REQUEST,
                    &format!("A field is not satisfied: {}", field.0),
                    None
                );
            }
            return base::response::internal_error(None);
        }
    }

    let user = state.app.db.user.document.get(
        bson::doc! {
            "email": &payload.email
        }
    ).await;
    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => {
            return base::response::error(StatusCode::FORBIDDEN, "Wrong email or password.", None);
        }
        Err(error) => {
            tracing::error!("{}\n{}", payload.email, error);
            return base::response::internal_error(None);
        }
    };

    match
        state.app.worker.verify_pass.send(VerifyPassRequest {
            password: payload.password,
            hash: user.password_hash,
        }).await
    {
        Ok(Some(true)) => {
            match user.deleted {
                None => {}
                Some(_) => {
                    return base::response::error(
                        StatusCode::FORBIDDEN,
                        "This account is pending for deletion, please recover this account.",
                        None
                    );
                }
            }
        }
        Ok(Some(false)) => {
            return base::response::error(StatusCode::FORBIDDEN, "Wrong email or password.", None);
        }
        _ => {
            tracing::error!("Verify password worker failure.");
            return base::response::internal_error(None);
        }
    }

    let (token, refresh) = state.app.jwt.generate_pair(user.user_id.clone());

    return match
        state.app.db.user.token.clone().create(user.user_id, token.0.identifier, token.0.exp).await
    {
        Ok(_) => {
            let token_cookie = cookies::construct("token", token.1, TOKEN_MAX_AGE);
            let refresh_cookie = cookies::construct("refresh", refresh.1, REFRESH_MAX_AGE);

            base::response::success(
                StatusCode::OK,
                Some(AppendHeaders(vec![(SET_COOKIE, token_cookie), (SET_COOKIE, refresh_cookie)]))
            )
        }
        Err(error) => {
            tracing::error!("{}\n{}", payload.email, error);
            base::response::internal_error(None)
        }
    };
}
