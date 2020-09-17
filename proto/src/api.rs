use std::collections::{HashMap, HashSet};

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
        pub doc: DocInfo,
    }
}

pub mod inbox {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ListResponse {
        pub count: u64,
        pub docs: Vec<DocInfo>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GetResponse {
        pub doc: DocInfo,
        pub labels: HashSet<Label>,
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
        pub doc: DocInfo,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchResponse {
        pub count: u64,
        pub docs: Vec<DocInfo>,
    }
}