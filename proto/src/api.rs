use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod auth {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AuthRequest {
        pub password: String,
    }
}

pub mod upload {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UploadResponse {
        pub id: String,
    }
}

pub mod inbox {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ListResponse {
        pub count: u64,
        pub docs: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GetResponse {
        pub id: String,
        pub uploaded: DateTime<Utc>,
        pub labels: HashSet<String>,
        pub properties: HashMap<String, String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ArchiveRequest {
        pub labels: HashSet<String>,
        pub properties: HashMap<String, String>,
    }
}

pub mod archive {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BundleResponse {
        pub id: String,
        //    created: DateTime<Utc>,
        //    modified: DateTime<Utc>,

        // Other metadata...
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchResponse {
        pub count: u64,
        pub docs: Vec<String>,
    }
}