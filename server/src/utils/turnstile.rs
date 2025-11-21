use std::time::Duration;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct TurnstileResult {
    pub success: bool,
    pub challenge_ts: String,
    pub hostname: String,
    pub error_codes: Vec<String>,
    pub action: String,
    pub cdata: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct Turnstile {
    secret: String,
    http_client: reqwest::Client,
    retries: usize,
}
impl Turnstile {
    pub fn default() -> Self {
        let turnstile_secret = std::env
            ::var("TURNSTILE_SECRET")
            .expect("TURNSTILE_SECRET must be set in .env file");

        Turnstile {
            secret: turnstile_secret,
            http_client: reqwest::Client
                ::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            retries: 3,
        }
    }

    /// More strict checks needed.
    pub async fn verify(&self, clientstile: String) -> Result<bool, ()> {
        if clientstile.len() > 2048 {
            return Ok(false);
        }

        let request_construct = self.http_client
            .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
            .form(
                &[
                    ("secret", &self.secret),
                    ("response", &clientstile),
                ]
            );

        let mut tries = 0;
        let response = loop {
            let request_instance = match request_construct.try_clone() {
                Some(instance) => instance,
                None => {
                    eprintln!("Turnstile: Bad request construct.");
                    break None;
                }
            };
            match request_instance.send().await {
                Ok(response) => {
                    match response.json::<TurnstileResult>().await {
                        Ok(response) => {
                            break Some(response);
                        }
                        Err(error) => {
                            eprintln!("Turnstile parsing: {}", error);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Turnstile verify request: {}", error);
                }
            }

            tries += 1;
            if tries >= self.retries {
                break None;
            }
        };

        return match response {
            Some(response) => Ok(response.success),
            None => Err(()),
        };
    }
}
