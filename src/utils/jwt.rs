use std::{ fs::File, io::Read, path::Path };

use jsonwebtoken::{ DecodingKey, EncodingKey, Header, Validation };
use nanoid::nanoid;
use serde::{ Deserialize, Serialize };

use crate::consts::{ JWT_VALIDATION_ALGORITHM, REFRESH_MAX_AGE, TOKEN_MAX_AGE };

#[derive(Clone, Serialize, Deserialize)]
pub enum TokenKind {
    AUTHENTICATION,
    REFRESH,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub identifier: String,
    pub kind: TokenKind,
    pub account_id: String,
    pub exp: u64,
}

pub struct Jwt {
    private_key: Option<EncodingKey>,
    public_key: DecodingKey,
}
impl Jwt {
    pub fn new() -> Self {
        Jwt {
            private_key: {
                if let Some(private_keyring) = quick_read("private.kc.pem") {
                    Some(EncodingKey::from_ec_pem(&private_keyring).unwrap())
                } else {
                    tracing::warn!(
                        "No private key for JWT installed. Any method calls with private key usage will result in a panic."
                    );
                    None
                }
            },
            public_key: {
                DecodingKey::from_ec_pem(
                    &quick_read("public.kc.pem").expect("Public key for JWT must be included.")
                ).expect("Public key for JWT must be included.")
            },
        }
    }

    /// Will panic if the private key is not provided.
    ///
    /// Returns a pair of key, one is the auth token, one is refresh token.
    pub fn generate_pair(
        &self,
        account_id: String
    ) -> ((TokenClaims, String), (TokenClaims, String)) {
        let identifier = nanoid!(10);
        let created_at = jsonwebtoken::get_current_timestamp();

        let token_claims = TokenClaims {
            identifier: identifier.clone(),
            kind: TokenKind::AUTHENTICATION,
            account_id: account_id.clone(),
            exp: created_at + TOKEN_MAX_AGE.as_secs(),
        };

        let token = jsonwebtoken::jws
            ::encode(
                &Header::new(JWT_VALIDATION_ALGORITHM),
                Some(&token_claims),
                self.private_key.as_ref().unwrap()
            )
            .unwrap();

        let refresh_claims = TokenClaims {
            identifier: identifier.clone(),
            kind: TokenKind::REFRESH,
            account_id: account_id.clone(),
            exp: created_at + REFRESH_MAX_AGE.as_secs(),
        };

        let refresh = jsonwebtoken::jws
            ::encode(
                &Header::new(JWT_VALIDATION_ALGORITHM),
                Some(&refresh_claims),
                self.private_key.as_ref().unwrap()
            )
            .unwrap();

        (
            (token_claims, format!("{}.{}.{}", token.protected, token.payload, token.signature)),
            (
                refresh_claims,
                format!("{}.{}.{}", refresh.protected, refresh.payload, refresh.signature),
            ),
        )
    }

    /// Any error happens during verification will return `None`.
    pub fn verify(&self, token: &str) -> Option<TokenClaims> {
        let data = jsonwebtoken::decode::<TokenClaims>(
            token,
            &self.public_key,
            &Validation::new(JWT_VALIDATION_ALGORITHM)
        );

        return match data {
            Ok(data) => { Some(data.claims) }
            Err(_) => None,
        };
    }
}

fn quick_read(name: &str) -> Option<Vec<u8>> {
    let mut keyring = vec![];
    if let Ok(mut reader) = File::open(Path::new(name)) {
        reader.read_to_end(&mut keyring).unwrap();
        return Some(keyring);
    }
    return None;
}
