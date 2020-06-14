use std::borrow::Borrow;
use std::ffi::OsString;
use std::str::FromStr;

use base58::{FromBase58, ToBase58};
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

use anyhow::{anyhow, Error};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct DocId(Uuid);

impl DocId {
    pub fn random() -> Self { Self(Uuid::new_v4()) }

    pub fn to_base58(&self) -> String { self.0.as_bytes().to_base58() }
}

impl FromStr for DocId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s
            .from_base58()
            .map_err(|_| anyhow!("Invalid document ID"))?;
        Ok(Self(Uuid::from_slice(&id)?))
    }
}

impl Serialize for DocId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl std::fmt::Display for DocId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_base58())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Document,
    Preview,
    Plaintext,
    Metadata,
    ProcessLog,
    Other { name: OsString },
}

impl Kind {
    pub fn other(name: impl Into<OsString>) -> Self { Self::Other { name: name.into() } }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub struct Label(String);

impl<S: Into<String>> From<S> for Label {
    fn from(s: S) -> Self { Self(s.into()) }
}

impl Borrow<String> for Label {
    fn borrow(&self) -> &String { &self.0 }
}

impl Borrow<str> for Label {
    fn borrow(&self) -> &str { &self.0 }
}
