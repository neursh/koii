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
                    println!("No private key for JWT installed.");
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

    /// Will panic if the private key is damaged.
    pub fn generate(&self, claims: TokenClaims) -> String {
        let token = jsonwebtoken::jws
            ::encode(
                &Header::new(Algorithm::ES256),
                Some(&claims),
                self.private_key.as_ref().unwrap()
            )
            .unwrap();

        format!("{}.{}.{}", token.protected, token.payload, token.signature)
    }

    /// Will panic if the public key is damaged.
    pub fn verify(&self, token: String) -> Option<TokenClaims> {
        let data = jsonwebtoken::decode::<TokenClaims>(
            token,
            &self.public_key,
            &Validation::new(Algorithm::ES256)
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
