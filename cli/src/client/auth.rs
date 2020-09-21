use anyhow::Result;
use reqwest::header::HeaderValue;
use reqwest::Url;

use crate::proto::api::auth::AuthRequest;

#[derive(Debug)]
pub enum Auth {
    Login {
        password: String,
    },

    ApiKey {
        username: String,
        password: String,
    },
}

impl Auth {
    pub fn login(password: String) -> Self {
        return Self::Login { password };
    }

    pub fn api_key(username: String, password: String) -> Self {
        return Self::ApiKey { username, password };
    }
}

pub(super) enum Session {
    Token {
        token: String,
    },
    ApiKey {
        username: String,
        password: String,
    },
}

impl Session {
    pub(super) async fn authenticate(auth: Auth, base_url: &Url, client: &reqwest::Client) -> Result<Self> {
        match auth {
            Auth::Login { password } => {
                let response = client.post(&format!("{}/auth/login", base_url))
                    .json(&AuthRequest { password })
                    .send().await?
                    .error_for_status()?;

                let token = response.headers().get(reqwest::header::AUTHORIZATION)
                    .expect("Token missing in response")
                    .to_str().expect("Token invalid");

                return Ok(Self::Token { token: token.to_string() });
            }

            Auth::ApiKey { username, password } => {
                return Ok(Self::ApiKey { username, password });
            }
        }
    }

    pub(super) async fn send(&mut self, request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let request = match self {
            Self::Token { token } => request.bearer_auth(token),
            Self::ApiKey { username, password } => request.basic_auth(username, Some(password)),
        };

        let response = request.send().await?;

        let token = response.headers().get(reqwest::header::AUTHORIZATION)
            .map(HeaderValue::to_str)
            .transpose()?;

        if let (Self::Token { token }, Some(update)) = (self, token) {
            *token = update.to_string();
        }

        return Ok(response);
    }
}
