use std::time::Duration;

use cookie_rs::{ Cookie, cookie::SameSite };

pub fn consturct(name: &str, value: String, max_age: Duration) -> String {
    Cookie::builder(name, value)
        .domain(".koii.space")
        .path("/")
        .max_age(max_age)
        .same_site(SameSite::Lax)
        .http_only(true)
        .secure(true)
        .build()
        .to_string()
}
