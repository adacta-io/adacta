use std::time::{Duration, SystemTime};

use anyhow::Result;
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Serialize, Deserialize};

use crate::config::AuthConfig;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub sub: String,
    pub exp: u64,
}

impl Claims {
    pub fn new(username: String, timeout: Duration) -> Self {
        return Self {
            sub: username,
            exp: (SystemTime::now() + timeout)
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time before epoch")
                .as_secs(),
        };
    }
}

#[derive(Debug)]
pub struct Token {
    pub username: String,
}

pub struct Authenticator {
    pub username: String,
    pub passhash: String,

    // pub secret: String,

    pub jwt_decoding_key: DecodingKey<'static>,
    pub jwt_encoding_key: EncodingKey,

    pub jwt_token_duration: Duration,
}

impl Authenticator {
    pub async fn from_config(config: AuthConfig) -> Result<Self> {
        // TODO: Add some sanity checks (empty values, ...)

        return Ok(Self {
            username: config.username,
            passhash: config.passhash,
            // secret: config.secret,

            jwt_decoding_key: DecodingKey::from_secret(config.secret.as_bytes()).into_static(),
            jwt_encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),

            jwt_token_duration: Duration::from_secs(60 * 60), // TODO: Make configurable
        });
    }

    pub async fn verify_token(&self, bearer: &str) -> Result<Token> {
        let data = jsonwebtoken::decode::<Claims>(bearer,
                                                  &self.jwt_decoding_key,
                                                  &jsonwebtoken::Validation::default())?;

        return Ok(Token {
            username: data.claims.sub,
        });
    }

    pub async fn sign_token(&self, token: &Token) -> Result<String> {
        let bearer = jsonwebtoken::encode(&jsonwebtoken::Header::default(),
                                          &Claims::new(token.username.to_owned(), self.jwt_token_duration),
                                          &self.jwt_encoding_key)?;

        return Ok(bearer);
    }

    pub async fn login(&self, username: &str, password: &str) -> Option<Token> {
        let user = username == self.username.as_str();
        let pass = pwhash::unix::verify(password.as_bytes(), &self.passhash);

        if !user || !pass {
            return None;
        }

        return Some(Token {
            username: username.to_owned(),
        });
    }
}