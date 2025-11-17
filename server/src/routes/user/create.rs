use serde::Deserialize;
use axum::{ Extension, Json, extract::State, http::StatusCode };
use mongodb::bson::{ self, doc };

use crate::{
    base::{ self, response::ResponseModel },
    database::users::UserDocument,
    routes::user::UserRoutesState,
    services::verify_email::VerifyEmailRequest,
    utils::{ checks::credentials_checks, middlewares::{ AuthorizationInfo, AuthorizationStatus } },
};
use nanoid::nanoid;

#[derive(Deserialize, Clone)]
pub struct CreatePayload {
    pub email: String,
    pub password: String,
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

    // Perform checks before processing.
    let payload_task = payload.clone();
    match
        tokio::task::spawn_blocking(move ||
            credentials_checks(&payload_task.email, &payload_task.password)
        ).await
    {
        Ok(result) => {
            if let Err(bad) = result {
                return bad;
            }
        }
        _ => {
            return base::response::internal_error(None);
        }
    }

    match state.app.database.users.get_one(doc! { "email": &payload.email }).await {
        Ok(user) => {
            if user.is_some() {
                return base::response::error(
                    StatusCode::CONFLICT,
                    "An account with the same email already exists.",
                    None
                );
            }
        }
        Err(error) => {
            return parse_db_fail(error);
        }
    }

    let password_hash = match state.app.services.hash_pass.send(payload.password).await {
        Ok(Some(hash)) => hash,
        _ => {
            return base::response::internal_error(None);
        }
    };

    let verify_code = nanoid!(64);

    if
        state.app.services.verify_email
            .send_ignore_result(VerifyEmailRequest {
                email: payload.email.clone(),
                verify_code: verify_code.clone(),
            }).await
            .is_err()
    {
        return base::response::internal_error(None);
    }

    let user = UserDocument {
        _id: nanoid!(48),
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
