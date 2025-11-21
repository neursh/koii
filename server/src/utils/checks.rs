use axum::http::StatusCode;

use crate::base::{ self, response::ResponseModel };

pub fn credentials(email: &str, password: &str) -> Result<(), ResponseModel> {
    if fast_chemail::parse_email(email).is_ok() {
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
