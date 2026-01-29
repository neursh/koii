use std::time::Duration;

use cookie_rs::{ Cookie, cookie::SameSite };

pub fn consturct(name: &str, value: String, max_age: i64) -> String {
    Cookie::builder(name, value)
        .domain(".koii.space")
        .path("/")
        .max_age(Duration::from_secs(max_age as u64))
        .same_site(SameSite::Lax)
        .http_only(true)
        .secure(true)
        .build()
        .to_string()
}
