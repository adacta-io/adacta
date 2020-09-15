use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::*;

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
        pub id: DocId,
    }
}

pub mod inbox {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ListResponse {
        pub count: u64,
        pub docs: Vec<DocId>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GetResponse {
        pub id: DocId,
        pub uploaded: DateTime<Utc>,
        pub labels: HashSet<Label>,
        pub properties: HashMap<String, String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ArchiveRequest {
        pub labels: HashSet<Label>,
        pub properties: HashMap<String, String>,
    }
}

pub mod archive {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BundleResponse {
        pub id: DocId,
        //    created: DateTime<Utc>,
        //    modified: DateTime<Utc>,

        // Other metadata...
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchResponse {
        pub count: u64,
        pub docs: Vec<DocId>,
    }
}