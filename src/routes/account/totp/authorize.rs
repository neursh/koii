use axum::{ Extension, Json, extract::State, http::StatusCode };
use mongodb::bson;
use nanoid::nanoid;
use serde::Deserialize;
use validator::Validate;

use crate::{
    base::{ self, response::ResponseModel },
    database::totp::code::TotpUsedCodeDocument,
    env::{ ACCOUNT_TOKEN_IDENTIFIER_LENGTH, MFA_UPGRADE_MAX_AGE },
    middlewares::auth::AuthorizationInfo,
    routes::account::AccountRoutesState,
    utils::jwt::KeyKind,
};

#[derive(Deserialize, Validate, Clone)]
pub struct UpgradePayload {
    #[validate(length(equal = 6))]
    pub totp_code: String,
    pub partial_login: Option<String>,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<AccountRoutesState>,
    Json(payload): Json<UpgradePayload>
) -> ResponseModel {
    match payload.validate() {
        Ok(_) => {}
        Err(_) => {
            return base::response::error(
                StatusCode::BAD_REQUEST,
                "TOTP code must be 6 characters.",
                None
            );
        }
    }

    let token = match payload.partial_login {
        Some(partial_login) => {
            let Some(token) = state.app.jwt.verify(&partial_login, KeyKind::PartialLogin) else {
                return base::response::error(StatusCode::UNAUTHORIZED, "Get out.", None);
            };
            token
        }
        None => {
            let Some(token) = authorization_info.token else {
                return base::response::error(StatusCode::UNAUTHORIZED, "Get out.", None);
            };
            token
        }
    };

    let totp = match state.app.db.totp.store.get_from_account(&token.account_id).await {
        Ok(Some(totp)) => totp,
        Ok(None) => {
            return base::response::error(
                StatusCode::NOT_FOUND,
                "No TOTP method was found for this account.",
                None
            );
        }
        Err(error) => {
            tracing::error!("Can't fetch TOTP struct for {}: {error}", &token.account_id);
            return base::response::internal_error(None);
        }
    };

    match totp.verify(&payload.totp_code) {
        Ok(true) => {}
        Ok(false) => {
            return base::response::error(StatusCode::FORBIDDEN, "Wrong TOTP code.", None);
        }
        Err(error) => {
            tracing::error!("Verify TOTP failed for {}: {error}", &token.account_id);
            return base::response::internal_error(None);
        }
    }

    let totp_used = TotpUsedCodeDocument {
        account_id: token.account_id,
        code: payload.totp_code,
        created_at: bson::DateTime::now(),
    };

    match state.app.db.totp.code.consume(&totp_used).await {
        Ok(true) => {}
        Ok(false) => {
            return base::response::error(StatusCode::FORBIDDEN, "Wrong TOTP code.", None);
        }
        Err(error) => {
            tracing::error!("Can't use TOTP code for {}: {error}", &totp_used.account_id);
            return base::response::internal_error(None);
        }
    }

    let identifier = nanoid!(*ACCOUNT_TOKEN_IDENTIFIER_LENGTH);
    let mfa_upgrade = state.app.jwt.generate(
        totp_used.account_id,
        identifier,
        KeyKind::MfaUpgrade,
        jsonwebtoken::get_current_timestamp() + MFA_UPGRADE_MAX_AGE.as_secs()
    );

    base::response::result(StatusCode::OK, mfa_upgrade.signed.into(), None)
}
