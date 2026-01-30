use std::time::Duration;

/// 15 days.
pub const TOKEN_MAX_AGE: Duration = Duration::from_hours(15 * 24);
/// 10 minutes.
pub const EMAIL_VERIFY_EXPIRE: Duration = Duration::from_mins(10);
