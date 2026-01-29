use serde::Deserialize;
use axum::{ Extension, Json, extract::State, http::StatusCode };
use mongodb::{ bson::{ self, doc }, error::WriteFailure };
use validator::Validate;

use crate::{
    base::{ self, response::ResponseModel },
    middlewares::auth::{ AuthorizationInfo, AuthorizationStatus },
    routes::user::UserRoutesState,
    store::users::UserDocument,
    workers::verify_email::VerifyEmailRequest,
};
use nanoid::nanoid;

#[derive(Deserialize, Validate, Clone)]
pub struct CreatePayload {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 12))]
    pub password: String,
    #[validate(length(max = 2048))]
    pub clientstile: String,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<UserRoutesState>,
    Json(payload): Json<CreatePayload>
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

    match state.app.turnstile.verify(payload.clientstile).await {
        Ok(true) => {}
        Ok(false) => {
            return base::response::error(
                StatusCode::BAD_REQUEST,
                "Something went wrong, try refresh the page and enter information again.",
                None
            );
        }
        Err(_) => {
            tracing::error!(target: "user.create", "Turnstile failure.");
            return base::response::internal_error(None);
        }
    }

    let user_id = nanoid!(48);
    let verify_code = nanoid!(64);
    let password_hash = match state.app.worker.hash_pass.send(payload.password).await {
        Ok(Some(hash)) => hash,
        _ => {
            tracing::error!(target: "user.create", "Hash password worker failure.");
            return base::response::internal_error(None);
        }
    };

    let user = UserDocument {
        id: user_id.clone(),
        email: payload.email.clone(),
        password_hash,
        verify_requested: Some(bson::DateTime::now()),
        verify_code: Some(verify_code.clone()),
        created_at: None,
    };

    match state.app.store.users.add(user).await {
        Ok(_) => {}
        Err(error) => {
            match *error.kind {
                mongodb::error::ErrorKind::Write(WriteFailure::WriteError(ref write_error)) if
                    write_error.code == 11000
                => {
                    return base::response::result(
                        StatusCode::CREATED,
                        "Check your inbox to verify your email!".into(),
                        None
                    );
                }
                _ => {
                    tracing::error!(target: "user.create", "Database failed to store a user: {}", error);
                    return base::response::internal_error(None);
                }
            }
        }
    }

    match
        state.app.worker.verify_email.send_ignore(VerifyEmailRequest {
            email: payload.email,
            verify_code,
        }).await
    {
        Ok(_) => {}
        Err(_) => {
            tracing::error!(target: "user.create", "Email worker failed to deliver the verification link for {}.", user_id);
            return base::response::internal_error(None);
        }
    }

    base::response::result(
        StatusCode::CREATED,
        "Check your inbox to verify your email!".into(),
        None
    )
}
