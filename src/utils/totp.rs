use std::time::SystemTimeError;

use mongodb::bson::Binary;
use serde::{ Deserialize, Serialize };
use totp_rs::{ SecretParseError, TOTP, TotpUrlError };
use thiserror::Error;

#[derive(Serialize, Deserialize)]
pub struct Totp {
    pub secret: Binary,
}

#[derive(Error, Debug)]
pub enum TotpError {
    #[error("TOTP malformed")] InvalidTotp(#[from] TotpUrlError),
    #[error("Secret malformed")] InvalidSecret(#[from] SecretParseError),
    #[error("Time error")] TimeError(#[from] SystemTimeError),
}

impl Totp {
    pub fn new() -> Self {
        Totp {
            secret: Binary {
                subtype: mongodb::bson::spec::BinarySubtype::Generic,
                bytes: nanoid::rngs::default(128),
            },
        }
    }

    pub fn url(&self, email: String) -> Result<String, TotpError> {
        let totp = TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            self.secret.clone().bytes,
            Some("Koii-Auth".to_string()),
            email
        )?;

        Ok(totp.get_url())
    }

    pub fn verify(&self, token: &str, email: String) -> Result<bool, TotpError> {
        let totp = TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            self.secret.clone().bytes,
            Some("Koii-Auth".to_string()),
            email
        )?;

        Ok(totp.check_current(token)?)
    }
}
