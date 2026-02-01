use std::time::Duration;

/// 15 minutes.
pub const TOKEN_MAX_AGE: Duration = Duration::from_mins(15);
/// 15 days.
pub const REFRESH_MAX_AGE: Duration = Duration::from_hours(15 * 24);
/// 1 minutes.
pub const SUDO_MAX_AGE: Duration = Duration::from_mins(1);
/// 10 minutes.
pub const EMAIL_VERIFY_EXPIRE: Duration = Duration::from_mins(10);
