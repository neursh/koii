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
    // Authentication status must not be `Authorized`.
    if let AuthorizationStatus::Authorized = authorization_info.status {
        return base::response::error(
            StatusCode::FORBIDDEN,
            "There's already an active user.",
            None
        );
    }

    // Validate before processing.
    match payload.validate() {
        Ok(_) => {} // valid, move on
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

    // Verify clientstile token.
    match state.app.turnstile.verify(payload.clientstile).await {
        Ok(true) => {} // valid, move on
        Ok(false) => {
            return base::response::error(
                StatusCode::BAD_REQUEST,
                "Something went wrong, try refresh the page and enter information again.",
                None
            );
        }
        Err(_) => {
            return base::response::internal_error(None);
        }
    }

    // Create a new user id.
    let user_id = nanoid!(48);
    let verify_code = nanoid!(64);

    // Hash the password.
    let password_hash = match state.app.worker.hash_pass.send(payload.password).await {
        Ok(Some(hash)) => hash,
        // Err & Ok(None)
        _ => {
            return base::response::internal_error(None);
        }
    };

    // Add the user to store (2nd pass).
    let user = UserDocument {
        id: user_id.clone(),
        email: payload.email.clone(),
        password_hash,
        verify_requested: Some(bson::DateTime::now()),
        verify_code: Some(verify_code.clone()),
        created_at: None,
        accept_refresh_after: None,
    };

    match state.app.store.users.add(user).await {
        Ok(_) => {} // valid, move on
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
                    tracing::error!(name: "user_store", "Database failed to store the user: {}", error.kind);
                    return base::response::internal_error(None);
                }
            }
        }
    }

    // Send a verification email to the user.
    match
        state.app.worker.verify_email.send_ignore(VerifyEmailRequest {
            email: payload.email,
            verify_code,
        }).await
    {
        Ok(_) => {}
        Err(_) => {
            tracing::error!({ user_id = user_id, },"Email service failed to deliver the verification link.");
            tracing::warn!(name: "email_service", "This indicates that the email service is down.");
            return base::response::internal_error(None);
        }
    }

    base::response::result(
        StatusCode::CREATED,
        "Check your inbox to verify your email!".into(),
        None
    )
}
