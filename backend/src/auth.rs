use std::time::{Duration, SystemTime};

use anyhow::Result;
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Serialize, Deserialize};

use crate::config::AuthConfig;
use std::collections::HashMap;

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
    username: String,
    passhash: String,

    // pub secret: String,

    jwt_decoding_key: DecodingKey<'static>,
    jwt_encoding_key: EncodingKey,

    jwt_token_duration: Duration,

    api_keys: HashMap<String, String>,
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

            api_keys: config.api_keys,
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
        let pass = bcrypt::verify(password.as_bytes(), &self.passhash)
            .expect("Can not verify password");

        if !user || !pass {
            return None;
        }

        return Some(Token {
            username: username.to_owned(),
        });
    }

    pub async fn verify_key(&self, username: &str, password: &str) -> Option<Token> {
        if bcrypt::verify(password, self.api_keys.get(username)?).ok()? {
            return Some(Token {
                username: username.to_owned(),
            });
        } else {
            return None;
        }
    }
}