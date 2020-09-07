use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use serde::export::PhantomData;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

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

pub trait BundleState {
    fn path(repository: &Repository) -> PathBuf;
}

#[async_trait]
pub trait BundleContainer<'r> {
    type State: BundleState;

    async fn get(&self, id: DocId) -> Option<Bundle<'r, Self::State>>;
}

pub struct Staging{}

impl BundleState for Staging {
    fn path(repository: &Repository) -> PathBuf {
        return repository.path.as_ref().as_ref().join("staging");
    }
}

pub struct Inboxed{}

impl BundleState for Inboxed {
    fn path(repository: &Repository) -> PathBuf {
        return repository.path.as_ref().as_ref().join("inbox");
    }
}

pub struct Archived{}

impl BundleState for Archived {
    fn path(repository: &Repository) -> PathBuf {
        return repository.path.as_ref().as_ref().join("archive");
    }
}

pub struct Bundle<'r, State: BundleState> {
    id: DocId,
    repository: &'r Repository,
    state: PhantomData<State>,
}

pub struct Repository {
    path: Box<dyn AsRef<Path> + Send + Sync>,
}

pub struct Inbox<'r>(&'r Repository);

#[async_trait]
impl<'r> BundleContainer<'r> for Inbox<'r> {
    type State = Inboxed;

    async fn get(&self, id: DocId) -> Option<Bundle<'r, Inboxed>> {
        let bundle = Bundle {
            id,
            repository: self.0,
            state: PhantomData::default(),
        };

        let metadata = tokio::fs::metadata(&bundle.path()).await;
        if metadata.is_err() {
            return None;
        }

        return Some(bundle);
    }
}

pub struct Archive<'r>(&'r Repository);

#[async_trait]
impl<'r> BundleContainer<'r> for Archive<'r> {
    type State = Archived;

    async fn get(&self, id: DocId) -> Option<Bundle<'r, Archived>> {
        let bundle = Bundle {
            id,
            repository: self.0,
            state: PhantomData::default(),
        };

        let metadata = tokio::fs::metadata(&bundle.path()).await;
        if metadata.is_err() {
            return None;
        }

        return Some(bundle);
    }
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

impl<State: BundleState> Bundle<'_, State> {
    pub fn id(&self) -> &DocId { return &self.id; }

    pub fn path(&self) -> PathBuf { return State::path(self.repository).join(self.id.filename()); }

    pub async fn fragment(&self, kind: Kind) -> Option<Fragment> {
        let path = self.path().join(kind.filename());

        let metadata = tokio::fs::metadata(&path).await;
        if metadata.is_err() {
            return None;
        }

        return Some(Fragment { kind, path });
    }

    pub async fn plaintext(&self) -> Option<Result<String>> {
        let fragment = self.fragment(Kind::Plaintext).await?;
        return Some(fragment.read_all().await);
    }

    pub async fn metadata(&self) -> Option<Result<Metadata>> {
        let fragment = self.fragment(Kind::Metadata).await?;

        return match fragment.read().await {
            Ok(r) => Some(Metadata::load(r).await),
            Err(err) => Some(Err(err)),
        };
    }
}

impl  Repository {
    pub async fn from_config(config: Config) -> Result<Self> {
        return Self::with_path(config.path).await;
    }

    pub async fn with_path(path: impl AsRef<Path> + Send + Sync + 'static) -> Result<Self> {
        // Create repository path if missing
        tokio::fs::create_dir_all(&path).await?;

        return Ok(Self { path: Box::new(path) });
    }

    pub fn path(&self) -> &Path { return self.path.as_ref().as_ref(); }

    pub fn inbox(&self) -> Inbox {
        return Inbox(self);
    }

    pub fn archive(&self) -> Archive {
        return Archive(self);
    }

    pub async fn stage(&self) -> Result<Bundle<'_, Staging>> {
        let bundle = Bundle {
            id: DocId::random(),
            repository: self,
            state: Default::default(),
        };

        tokio::fs::create_dir_all(&bundle.path()).await?;

        return Ok(bundle);
    }
}

impl <'r> Bundle<'r, Inboxed> {
    pub async fn archive(self) -> Result<Bundle<'r, Archived>> {
        let archived = Bundle {
            id: self.id,
            repository: self.repository,
            state: PhantomData::default(),
        };

        tokio::fs::create_dir_all(archived.path().parent().expect("No parent directory")).await?;
        tokio::fs::rename(&self.path(), &archived.path()).await?;

        return Ok(archived);
    }

    pub async fn delete(self) -> Result<()> {
        tokio::fs::remove_dir_all(&self.path()).await?;

        return Ok(());
    }
}

impl <'r> Bundle<'r, Staging> {
    pub async fn create(self) -> Result<Bundle<'r, Inboxed>> {
        let inboxed = Bundle {
            id: self.id,
            repository: self.repository,
            state: PhantomData::default(),
        };

        tokio::fs::create_dir_all(inboxed.path().parent().expect("No parent directory")).await?;
        tokio::fs::rename(&self.path(), &inboxed.path()).await?;

        return Ok(inboxed);
    }

    pub async fn write(&self, kind: Kind) -> Result<impl AsyncWrite> {
        let path = self.path().join(kind.filename());

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await?;

        return Ok(file);
    }

    pub async fn delete(self) -> Result<()> {
        tokio::fs::remove_dir_all(&self.path()).await?;

        return Ok(());
    }
}

impl <'r> Bundle<'r, Inboxed> {
    pub async fn write_manifest(&self) -> Result<impl AsyncRead + AsyncWrite> {
        let path = self.path().join(Kind::Metadata.filename());

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await?;

        return Ok(file);
    }
}
