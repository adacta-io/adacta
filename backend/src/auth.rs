use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};

use anyhow::Result;
use jsonwebtoken::{DecodingKey, EncodingKey};

use crate::config::Auth;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub exp: u64,
}

impl Claims {
    pub fn new(timeout: Duration) -> Self {
        return Self {
            exp: (SystemTime::now() + timeout)
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time before epoch")
                .as_secs(),
        };
    }
}

#[derive(Debug)]
pub struct Token {
    private: (),
}

pub struct Authenticator {
    passhash: String,

    jwt_decoding_key: DecodingKey<'static>,
    jwt_encoding_key: EncodingKey,

    jwt_token_duration: Duration,

    api_keys: HashMap<String, String>,
}

impl Authenticator {
    pub async fn from_config(config: Auth) -> Result<Self> {
        // TODO: Add some sanity checks (empty values, ...)

        Ok(Self {
            passhash: config.passhash,

            jwt_decoding_key: DecodingKey::from_secret(config.secret.as_bytes()).into_static(),
            jwt_encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),

            jwt_token_duration: Duration::from_secs(60 * 60), // TODO: Make configurable

            api_keys: config.api_keys,
        })
    }

    pub async fn verify_token(&self, bearer: &str) -> Result<Token> {
        jsonwebtoken::decode::<Claims>(
            bearer,
            &self.jwt_decoding_key,
            &jsonwebtoken::Validation::default(),
        )?;

        Ok(Token { private: () })
    }

    pub async fn sign_token(&self, _: &Token) -> Result<String> {
        let bearer = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &Claims::new(self.jwt_token_duration),
            &self.jwt_encoding_key,
        )?;

        Ok(bearer)
    }

    pub async fn login(&self, password: &str) -> Option<Token> {
        // TODO: Verify passhash is valid on config load

        if bcrypt::verify(password.as_bytes(), &self.passhash).ok()? {
            return Some(Token { private: () });
        } else {
            return None;
        }
    }

    pub async fn verify_key(&self, username: &str, password: &str) -> Option<Token> {
        if bcrypt::verify(password, self.api_keys.get(username)?).ok()? {
            return Some(Token { private: () });
        } else {
            return None;
        }
    }
}
