use std::{ fs::File, io::Read, path::Path };

use jsonwebtoken::{ Algorithm, DecodingKey, EncodingKey, Header, Validation };
use serde::{ Deserialize, Serialize };

#[derive(Clone, Serialize, Deserialize)]
pub enum TokenUsage {
    Authorize,
    Refresh,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub usage: TokenUsage,
    pub id: String,
    pub exp: i64,
}

#[derive(Clone)]
pub struct Jwt {
    private_key: Option<EncodingKey>,
    public_key: Option<DecodingKey>,
}
impl Jwt {
    pub fn new() -> Self {
        Jwt {
            private_key: {
                if let Some(private_keyring) = quick_read("private.kc.pem") {
                    Some(EncodingKey::from_ec_pem(&private_keyring).unwrap())
                } else {
                    None
                }
            },
            public_key: {
                if let Some(public_keyring) = quick_read("public.kc.pem") {
                    Some(DecodingKey::from_ec_pem(&public_keyring).unwrap())
                } else {
                    None
                }
            },
        }
    }

    pub fn generate(&self, claims: TokenClaims) -> Result<String, ()> {
        if
            let Some(private_key) = &self.private_key &&
            let Ok(token) = jsonwebtoken::jws::encode(
                &Header::new(Algorithm::ES256),
                Some(&claims),
                private_key
            )
        {
            return Ok(format!("{}.{}.{}", token.protected, token.payload, token.signature));
        }
        Err(())
    }

    pub fn verify(&self, token: String) -> Result<TokenClaims, ()> {
        if let Some(public_key) = &self.public_key {
            let data = jsonwebtoken::decode::<TokenClaims>(
                token,
                public_key,
                &Validation::new(Algorithm::ES256)
            );

            match data {
                Ok(data) => {
                    return Ok(data.claims);
                }
                _ => {
                    return Err(());
                }
            }
        }
        Err(())
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
