use serde::Deserialize;
use axum::{ Extension, Json, extract::State, http::StatusCode };
use mongodb::bson::{ self, doc };
use validator::Validate;

use crate::{
    base::{ self, response::ResponseModel },
    database::users::UserDocument,
    routes::user::UserRoutesState,
    worker::verify_email::VerifyEmailRequest,
    utils::{ cookie_query::{ AuthorizationInfo, AuthorizationStatus } },
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

    // Vro I don't like too much processing yk.
    if state.semaphores.create.acquire().await.is_err() {
        return base::response::internal_error(None);
    }

    // Validate before processing.
    match payload.validate() {
        Ok(_) => {} // valid, move on
        Err(error) => {
            let (bad_field, _) = error.errors().iter().next().unwrap();
            if bad_field == "email" {
                return base::response::error(StatusCode::BAD_REQUEST, "Invalid email.", None);
            }
            if bad_field == "password" {
                return base::response::error(
                    StatusCode::BAD_REQUEST,
                    "Password must be longer than 12 characters.",
                    None
                );
            }
            if bad_field == "clientstile" {
                return base::response::error(
                    StatusCode::BAD_REQUEST,
                    "Invalid turnstile token length u cheeky lad UvU",
                    None
                );
            }

            return base::response::internal_error(None);
        }
    }

    // Check turnstile token.
    match state.app.turnstile.verify(payload.clientstile).await {
        Ok(true) => {} // valid, move on
        Ok(false) => {
            return base::response::error(
                StatusCode::BAD_REQUEST,
                "Invalid turnstile token length u cheeky lad UvU",
                None
            );
        }
        Err(_) => {
            return base::response::internal_error(None);
        }
    }

    // Check if the email is already used.
    match state.app.database.users.get_one(doc! { "email": &payload.email }).await {
        Ok(None) => {} // valid, move on
        Ok(Some(_)) => {
            return base::response::error(
                StatusCode::CONFLICT,
                "An account with the same email already exists.",
                None
            );
        }
        Err(error) => {
            return parse_db_fail(error);
        }
    }

    let password_hash = match state.app.worker.hash_pass.send(payload.password).await {
        Ok(Some(hash)) => hash,
        // Err & Ok(None)
        _ => {
            return base::response::internal_error(None);
        }
    };

    let verify_code = nanoid!(64);

    if
        state.app.worker.verify_email
            .send_ignore_result(VerifyEmailRequest {
                email: payload.email.clone(),
                verify_code: verify_code.clone(),
            }).await
            .is_err()
    {
        return base::response::internal_error(None);
    }

    let user = UserDocument {
        id: nanoid!(48),
        email: payload.email,
        password_hash,
        verify_requested: Some(bson::DateTime::now()),
        verify_code: Some(verify_code),
        created_at: None,
        accept_refresh_after: None,
    };

    match state.app.database.users.add(user).await {
        Ok(_) => base::response::success(StatusCode::CREATED, None),
        Err(error) => parse_db_fail(error),
    }
}

fn parse_db_fail(error: mongodb::error::Error) -> ResponseModel {
    use mongodb::error::{ ErrorKind, WriteFailure };

    match *error.kind {
        ErrorKind::Write(WriteFailure::WriteError(ref write_error)) if write_error.code == 11000 => {
            base::response::error(
                StatusCode::BAD_REQUEST,
                "An account with the same email already exists.",
                None
            )
        }
        _ => {
            eprintln!("Database error: {:?}", error.kind);
            return base::response::internal_error(None);
        }
    }
}
