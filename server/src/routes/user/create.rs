use serde::Deserialize;
use axum::{ Extension, Json, extract::State, http::StatusCode };
use mongodb::bson::{ self, doc };

use crate::{
    base::{ self, response::ResponseModel },
    database::users::UserDocument,
    routes::user::RouteState,
    services::verify_email::VerifyEmailRequest,
    utils::middlewares::{ AuthorizationInfo, AuthorizationStatus },
};
use nanoid::nanoid;

#[derive(Deserialize, Clone)]
pub struct CreatePayload {
    pub email: String,
    pub password: String,
}

pub async fn handler(
    Extension(authorization_info): Extension<AuthorizationInfo>,
    State(state): State<RouteState>,
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
    match tokio::task::spawn_blocking(|| payload_checks(payload_task)).await {
        Ok(result) => {
            if let Err(bad) = result {
                return bad;
            }
        }
        _ => {
            return base::response::internal_error(None);
        }
    }

    match state.koii_database.users.get_one(doc! { "email": &payload.email }).await {
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

    let password_hash = match state.services.hash_pass.send(payload.password).await {
        Ok(Some(hash)) => hash,
        _ => {
            return base::response::internal_error(None);
        }
    };

    let verify_code = nanoid!(64);

    if
        state.services.verify_email
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
    };

    match state.koii_database.users.add(user).await {
        Ok(_) => base::response::success(StatusCode::CREATED, None),
        Err(error) => parse_db_fail(error),
    }
}

fn payload_checks(payload: CreatePayload) -> Result<(), ResponseModel> {
    if !mailchecker::is_valid(&payload.email) {
        return Err(base::response::error(StatusCode::BAD_REQUEST, "Invalid email provided.", None));
    }

    if payload.password.len() < 8 {
        return Err(
            base::response::error(
                StatusCode::BAD_REQUEST,
                "Password must be longer than 8 characters.",
                None
            )
        );
    }

    Ok(())
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
