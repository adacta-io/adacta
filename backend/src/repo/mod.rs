use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

use async_trait::async_trait;

use crate::config::Repository as Config;
use crate::meta::Metadata;
use crate::model::{DocId, Kind};

trait Filename {
    fn filename(&self) -> OsString;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fragment {
    kind: Kind,
    path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bundle {
    id: DocId,
    path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    path: PathBuf,
}

impl Filename for Kind {
    fn filename(&self) -> OsString {
        return match self {
            Self::Document => OsString::from("document.pdf"),
            Self::Preview => OsString::from("preview.png"),
            Self::Plaintext => OsString::from("document.txt"),
            Self::Metadata => OsString::from("metadata.json"),
            Self::ProcessLog => OsString::from("process.log"),
            Self::Other { name } => OsString::from(name),
        };
    }
}

impl Filename for DocId {
    fn filename(&self) -> OsString { return self.to_string().into(); }
}

impl Fragment {
    pub fn kind(&self) -> &Kind { return &self.kind; }

    pub fn path(&self) -> &Path { return self.path.as_path(); }

    //    pub async fn exists(&self) -> bool {
    //        return self.path.exists().await;
    //    }

    pub async fn read(&self) -> Result<impl AsyncRead> {
        let file = OpenOptions::new().read(true).open(&self.path).await?;

        return Ok(file);
    }

    pub async fn read_all(&self) -> Result<String> {
        let mut r = self.read().await?;

        let mut buffer = String::new();
        r.read_to_string(&mut buffer).await?;

        return Ok(buffer);
    }
}

#[async_trait]
pub trait FragmentContainer {
    fn id(&self) -> &DocId;

    fn path(&self) -> &Path;

    async fn fragment(&self, kind: Kind) -> Option<Fragment> {
        let path = self.path().join(kind.filename());

        let metadata = tokio::fs::metadata(&path).await;
        if metadata.is_err() {
            return None;
        }

        return Some(Fragment { kind, path });
    }

    async fn plaintext(&self) -> Option<Result<String>> {
        let fragment = self.fragment(Kind::Plaintext).await?;
        return Some(fragment.read_all().await);
    }

    async fn metadata(&self) -> Option<Result<Metadata>> {
        let fragment = self.fragment(Kind::Metadata).await?;

        return match fragment.read().await {
            Ok(r) => Some(Metadata::load(r).await),
            Err(err) => Some(Err(err)),
        };
    }
}

#[async_trait]
impl FragmentContainer for Bundle {
    fn id(&self) -> &DocId { return &self.id; }

    fn path(&self) -> &Path { return self.path.as_path(); }
}

impl Repository {
    pub async fn from_config(config: Config) -> Result<Self> {
        let path = PathBuf::from(config.path);

        // Create repository path if missing
        tokio::fs::create_dir_all(&path).await?;

        return Ok(Self { path });
    }

    pub fn path(&self) -> &Path { return self.path.as_path(); }

    //    pub fn list(&self) -> Result<Vec<Bundle>> {
    //        return std::fs::read_dir(&self.path)?
    //            .map(|entry| {
    //                let entry = entry?;
    //
    //                let id = entry.file_name().to_string_lossy().parse()?;
    //
    //                return Ok(Bundle {
    //                    id,
    //                    path: entry.path(),
    //                });
    //            })
    //            .collect();
    //    }

    pub async fn get(&self, id: DocId) -> Option<Bundle> {
        let path = self.path.join(id.filename());

        let metadata = tokio::fs::metadata(&path).await;
        if metadata.is_err() {
            return None;
        }

        return Some(Bundle { id, path });
    }

    pub async fn stage(&self) -> Result<BundleStaging> {
        let id = DocId::random();
        let path = self.path.join("staging").join(id.filename());

        tokio::fs::create_dir_all(&path).await?;

        return Ok(BundleStaging { id, path });
    }
}

pub struct BundleStaging {
    id: DocId,
    path: PathBuf,
}

#[async_trait]
impl FragmentContainer for BundleStaging {
    fn id(&self) -> &DocId { return &self.id; }

    fn path(&self) -> &Path { return self.path.as_path(); }
}

impl BundleStaging {
    pub async fn create(self, repo: &Repository) -> Result<Bundle> {
        let target_path = repo.path.join(self.id.filename());

        tokio::fs::rename(&self.path, &target_path).await?;

        return Ok(Bundle {
            id: self.id,
            path: target_path,
        });
    }

    pub async fn write(&self, kind: Kind) -> Result<impl AsyncWrite> {
        let path = self.path.join(kind.filename());

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await?;

        return Ok(file);
    }
}
