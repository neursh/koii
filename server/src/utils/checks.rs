use axum::http::StatusCode;

use crate::base::{ self, response::ResponseModel };

pub fn credentials_checks(email: &str, password: &str) -> Result<(), ResponseModel> {
    if !mailchecker::is_valid(email) {
        return Err(base::response::error(StatusCode::BAD_REQUEST, "Invalid email provided.", None));
    }

    if password.len() < 8 {
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
