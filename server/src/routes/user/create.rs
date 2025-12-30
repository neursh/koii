use std::borrow::Cow::Borrowed;
use serde::Deserialize;
use axum::{ Extension, Json, extract::State, http::StatusCode };
use mongodb::{ bson::{ self, doc }, error::WriteFailure };
use validator::Validate;

use crate::{
    base::{ self, response::ResponseModel },
    cache::verify::VerifyCacheQuery,
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
impl CreatePayload {
    fn check(&self) -> Result<(), &str> {
        if let Err(error) = self.validate() {
            match error.errors().iter().next() {
                Some((bad_field, _)) => {
                    match bad_field {
                        Borrowed("email") => {
                            return Err("Invalid email.");
                        }
                        Borrowed("password") => {
                            return Err("Password must be longer than 12 characters.");
                        }
                        Borrowed("clientstile") => {
                            return Err("Something went wrong, refresh the page and try again.");
                        }
                        _ => {
                            return Err("unknown");
                        }
                    }
                }
                None => {
                    return Err("unknown");
                }
            }
        }

        Ok(())
    }
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

    // Accquire a semaphore permit.
    let permit = match state.semaphores.create.acquire().await {
        Ok(permit) => permit,
        Err(_) => {
            return base::response::internal_error(None);
        }
    };

    // Validate before processing.
    match payload.check() {
        Ok(_) => {} // valid, move on
        Err(message) => {
            return base::response::error(StatusCode::BAD_REQUEST, message, None);
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

    // Check if email is used before hashing the password to avoid excessive computes.
    match state.app.store.users.get_one(doc! { "email": &payload.email }).await {
        Ok(None) => {} // valid, move on
        Ok(Some(_)) => {
            return base::response::error(
                StatusCode::CONFLICT,
                "An account with the same email already exists.",
                None
            );
        }
        Err(error) => {
            eprintln!("Database error: {:?}", error);
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
                    return base::response::error(
                        StatusCode::BAD_REQUEST,
                        "An account with the same email already exists.",
                        None
                    );
                }
                _ => {
                    eprintln!("Database error: {:?}", error.kind);
                    return base::response::internal_error(None);
                }
            }
        }
    }

    // Send a verification email to the user.
    if
        state.app.worker.verify_email
            .send_ignore(VerifyEmailRequest {
                email: payload.email.clone(),
                verify_code: verify_code.clone(),
            }).await
            .is_err()
    {
        return base::response::internal_error(None);
    }

    // Cache the verify code.
    match
        state.app.cache.verify.clone().add(VerifyCacheQuery {
            user_id: user_id.clone(),
            code: verify_code,
        }).await
    {
        Ok(_) => {}
        Err(error) => {
            eprintln!("Database error: {:?}", error);
            return base::response::internal_error(None);
        }
    }

    // Dropping semaphore permit, we done here.
    drop(permit);

    return base::response::success(StatusCode::CREATED, None);
}
