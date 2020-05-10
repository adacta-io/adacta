use std::ffi::OsString;
use std::str::FromStr;

use anyhow::{Error, anyhow};
use uuid::Uuid;
use base58::{FromBase58, ToBase58};
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct DocId(Uuid);

impl DocId {
    pub fn random() -> Self {
        return Self(Uuid::new_v4());
    }

    pub fn to_base58(&self) -> String {
        return self.0.as_bytes().to_base58();
    }
}

impl FromStr for DocId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.from_base58()
            .map_err(|_| anyhow!("Invalid document ID"))?;
        let id = Uuid::from_slice(&id)?;
        return Ok(DocId(id));
    }
}

impl Serialize for DocId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        return serializer.serialize_str(&self.to_string());
    }
}

impl std::fmt::Display for DocId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_str(&self.to_base58());
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
    pub fn other(name: impl Into<OsString>) -> Self {
        return Self::Other { name: name.into() };
    }
}