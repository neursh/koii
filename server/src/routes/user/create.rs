use serde::Deserialize;
use axum::{ Json, extract::State, http::StatusCode };
use mongodb::bson::{ self, doc };

use crate::{
    base::{ self, response::ResponseModel },
    database::users::UserDocument,
    routes::user::RouteState,
    services::verify_email::VerifyEmailRequest,
};
use nanoid::nanoid;

#[derive(Deserialize, Clone)]
pub struct CreatePayload {
    pub email: String,
    pub password: String,
}

pub async fn handler(
    State(state): State<RouteState>,
    Json(payload): Json<CreatePayload>
) -> (StatusCode, Json<ResponseModel>) {
    // Vro I don't like too much processing yk.
    if state.semaphores.create.acquire().await.is_err() {
        return base::response::internal_error();
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
            return base::response::internal_error();
        }
    }

    match state.koii_database.users.get_one(doc! { "email": &payload.email }).await {
        Ok(user) => {
            if user.is_some() {
                return base::response::error(
                    StatusCode::CONFLICT,
                    "An account with the same email already exists."
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
            return base::response::internal_error();
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
        return base::response::internal_error();
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
        Ok(_) => base::response::success(StatusCode::CREATED),
        Err(error) => parse_db_fail(error),
    }
}

fn payload_checks(payload: CreatePayload) -> Result<(), (StatusCode, Json<ResponseModel>)> {
    if !mailchecker::is_valid(&payload.email) {
        return Err(base::response::error(StatusCode::BAD_REQUEST, "Invalid email provided."));
    }

    if payload.password.len() < 8 {
        return Err(
            base::response::error(
                StatusCode::BAD_REQUEST,
                "Password must be longer than 8 characters."
            )
        );
    }

    Ok(())
}

fn parse_db_fail(error: mongodb::error::Error) -> (StatusCode, Json<ResponseModel>) {
    use mongodb::error::{ ErrorKind, WriteFailure };

    match *error.kind {
        ErrorKind::Write(WriteFailure::WriteError(ref write_error)) if write_error.code == 11000 => {
            base::response::error(
                StatusCode::BAD_REQUEST,
                "An account with the same email already exists."
            )
        }
        _ => {
            eprintln!("Database error: {:?}", error.kind);
            return base::response::internal_error();
        }
    }
}
