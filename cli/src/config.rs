use std::fs::File;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::client;
use crate::output::Output;

#[derive(Debug, Serialize, Deserialize)]
pub enum Auth {
    #[serde(rename = "login")]
    Login {
        #[serde(rename = "password")]
        password: String,
    },

    #[serde(rename = "apiKey")]
    ApiKey {
        #[serde(rename = "username")]
        username: String,

        #[serde(rename = "password")]
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

impl Into<client::Auth> for Auth {
    fn into(self) -> client::Auth {
        return match self {
            Self::Login { password } => client::Auth::login(password),
            Self::ApiKey { username, password } => client::Auth::api_key(username, password),
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "url")]
    pub url: String,

    #[serde(flatten)]
    pub auth: Auth,
}

impl Config {
    pub fn load(path: Option<impl AsRef<Path>>) -> Result<Self> {
        // TODO: Missing config file should bubble up as error
        let path = path.map(|path| path.as_ref().to_path_buf())
            .unwrap_or_else(||
                xdg::BaseDirectories::with_prefix("adacta")
                    .expect("Failed to open config directory")
                    .find_config_file("config.yaml")
                    .expect("Config file not found"));

        let file = File::open(path)?;
        let result = serde_yaml::from_reader(file)?;

        return Ok(result);
    }

    pub fn with(url: String, auth: Auth) -> Self {
        return Self { url, auth };
    }

    pub fn save(&self, path: Option<impl AsRef<Path>>) -> Result<()> {
        let path = path.map(|path| path.as_ref().to_path_buf())
            .unwrap_or_else(||
                xdg::BaseDirectories::with_prefix("adacta")
                    .expect("Failed to open config directory")
                    .place_config_file("config.yaml")
                    .expect("Config file not writeable"));

        serde_yaml::to_writer(File::create(path)?, self)?;

        return Ok(());
    }
}

pub fn exec(matches: &clap::ArgMatches<'_>) -> Result<Box<dyn Output>> {
    let target = matches.value_of("target").expect("Target required");
    let username = matches.value_of("username");
    let password = matches.value_of("password").expect("Password required");

    let auth = if let Some(username) = username {
        Auth::api_key(username.to_string(), password.to_string())
    } else {
        Auth::login(password.to_string())
    };

    let config = Config::with(target.to_owned(), auth);
    config.save(matches.value_of("config"))?;

    return Ok(Box::new(()));
}