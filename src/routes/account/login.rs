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
    routes::account::AccountRoutesState,
    utils::cookies,
    workers::verify_pass::VerifyPassRequest,
};

#[derive(Deserialize, Validate, Clone)]
pub struct LoginPayload {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 12))]
    pub password: String,
    #[validate(length(equal = 6))]
    pub totp_code: Option<String>,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<AccountRoutesState>,
    Json(payload): Json<LoginPayload>
) -> ResponseModel {
    if let AuthorizationStatus::Authorized = authorization_info.status {
        return base::response::error(
            StatusCode::FORBIDDEN,
            "There's already an active account.",
            None
        );
    }

    match payload.validate() {
        Ok(_) => {} // Valid payload, passing down.
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

    let account = state.app.db.account.document.get(
        bson::doc! {
            "email": &payload.email
        }
    ).await;
    let account = match account {
        Ok(Some(account)) => account,
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
            hash: account.password_hash,
        }).await
    {
        Ok(Some(true)) => {
            match account.deleted {
                None => {} // Account not marked for deletion, passing down.
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

    match account.totp {
        None => {} // No 2FA setup detected, passing down.
        Some(totp) => {
            let totp_code = match payload.totp_code {
                Some(totp_code) => { totp_code }
                None => {
                    return base::response::result(
                        StatusCode::ACCEPTED,
                        "TOTP Required".into(),
                        None
                    );
                }
            };

            match totp.verify(&totp_code) {
                Ok(true) => {} // Correct token, passing down.
                Ok(false) => {
                    return base::response::error(
                        StatusCode::UNAUTHORIZED,
                        "The TOTP code provided was wrong.",
                        None
                    );
                }
                Err(_) => {
                    return base::response::internal_error(None);
                }
            }
        }
    }

    let (token, refresh) = state.app.jwt.generate_pair(account.account_id.clone());

    return match
        state.app.db.account.token
            .clone()
            .create(account.account_id, token.0.identifier, token.0.exp).await
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
