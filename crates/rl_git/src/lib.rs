//! Git plumbing adapter layer for repo-lens.
//!
//! This crate abstracts Git implementation details (libgit2, gitoxide, subprocess)
//! behind a stable trait interface.

pub mod backend;

use rl_api::Error;
use std::path::Path;

// Re-export the CLI backend
pub use backend::CliBackend;

/// Result type for Git operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Git backend trait that abstracts the underlying Git implementation.
#[async_trait::async_trait]
pub trait GitBackend: Send + Sync {
    /// Open a repository at the given path.
    async fn open_repo(&self, path: &Path) -> Result<Box<dyn RepoHandle>>;

    /// Check if a path is a valid Git repository.
    async fn is_repo(&self, path: &Path) -> Result<bool>;
}

/// Handle to an open repository.
#[async_trait::async_trait]
pub trait RepoHandle: Send + Sync {
    /// Get a snapshot of the current repository state.
    async fn snapshot(&self) -> Result<RepoSnapshot>;

    /// Get the object store.
    fn object_store(&self) -> &dyn ObjectStore;

    /// Get the refs store.
    fn refs_store(&self) -> &dyn RefsStore;

    /// Get the working directory interface.
    fn workdir(&self) -> &dyn Workdir;

    /// Get the index reader.
    fn index_reader(&self) -> &dyn IndexReader;

    /// Get diff name-status between two revisions.
    async fn diff_name_status(&self, range: &str) -> Result<String>;

    /// Get diff numstat between two revisions.
    async fn diff_numstat(&self, range: &str) -> Result<String>;
}

/// Immutable snapshot of repository state at a point in time.
#[derive(Debug, Clone)]
pub struct RepoSnapshot {
    /// Repository path
    pub path: std::path::PathBuf,
    /// HEAD commit ID
    pub head: Option<String>,
    /// Current branch name
    pub branch: Option<String>,
    /// All references
    pub refs: Vec<RefInfo>,
}

/// Reference information.
#[derive(Debug, Clone)]
pub struct RefInfo {
    /// Reference name (e.g., "refs/heads/main")
    pub name: String,
    /// Target commit ID
    pub target: String,
    /// Whether this is a symbolic reference
    pub is_symbolic: bool,
}

/// Object store interface.
#[async_trait::async_trait]
pub trait ObjectStore: Send + Sync {
    /// Read a commit object.
    async fn read_commit(&self, id: &str) -> Result<Commit>;

    /// Read a tree object.
    async fn read_tree(&self, id: &str) -> Result<Tree>;

    /// Read a blob object.
    async fn read_blob(&self, id: &str) -> Result<Blob>;
}

/// Commit object.
#[derive(Debug, Clone)]
pub struct Commit {
    /// Commit ID
    pub id: String,
    /// Tree ID
    pub tree_id: String,
    /// Parent commit IDs
    pub parent_ids: Vec<String>,
    /// Author information
    pub author: Signature,
    /// Committer information
    pub committer: Signature,
    /// Commit message
    pub message: String,
}

/// Tree object.
#[derive(Debug, Clone)]
pub struct Tree {
    /// Tree ID
    pub id: String,
    /// Tree entries
    pub entries: Vec<TreeEntry>,
}

/// Tree entry.
#[derive(Debug, Clone)]
pub struct TreeEntry {
    /// Entry mode
    pub mode: u32,
    /// Entry name
    pub name: String,
    /// Object ID
    pub id: String,
    /// Entry type
    pub entry_type: TreeEntryType,
}

/// Tree entry type.
#[derive(Debug, Clone)]
pub enum TreeEntryType {
    /// Blob (file)
    Blob,
    /// Tree (directory)
    Tree,
    /// Commit (submodule)
    Commit,
}

/// Blob object.
#[derive(Debug, Clone)]
pub struct Blob {
    /// Blob ID
    pub id: String,
    /// Blob content
    pub content: Vec<u8>,
}

/// Signature (author/committer info).
#[derive(Debug, Clone)]
pub struct Signature {
    /// Name
    pub name: String,
    /// Email
    pub email: String,
    /// Timestamp
    pub time: i64,
}

/// References store interface.
#[async_trait::async_trait]
pub trait RefsStore: Send + Sync {
    /// Get all references.
    async fn all_refs(&self) -> Result<Vec<RefInfo>>;

    /// Resolve a reference to its target.
    async fn resolve_ref(&self, name: &str) -> Result<String>;
}

/// Working directory interface.
#[async_trait::async_trait]
pub trait Workdir: Send + Sync {
    /// Get status of the working directory.
    async fn status(&self) -> Result<WorkdirStatus>;
}

/// Working directory status.
#[derive(Debug, Clone)]
pub struct WorkdirStatus {
    /// Modified files
    pub modified: Vec<String>,
    /// Added files
    pub added: Vec<String>,
    /// Deleted files
    pub deleted: Vec<String>,
    /// Renamed files (old_name, new_name)
    pub renamed: Vec<(String, String)>,
    /// Untracked files
    pub untracked: Vec<String>,
}

/// Index reader interface.
#[async_trait::async_trait]
pub trait IndexReader: Send + Sync {
    /// Get all staged entries.
    async fn staged_entries(&self) -> Result<Vec<IndexEntry>>;
}

/// Index entry.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// File path
    pub path: String,
    /// Object ID
    pub id: String,
    /// File mode
    pub mode: u32,
}

// Stub implementation for scaffolding

/// Stub Git backend that returns "not implemented" errors.
pub struct StubGitBackend;

#[async_trait::async_trait]
impl GitBackend for StubGitBackend {
    async fn open_repo(&self, _path: &Path) -> Result<Box<dyn RepoHandle>> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }

    async fn is_repo(&self, _path: &Path) -> Result<bool> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }
}

/// Stub repository handle.
pub struct StubRepoHandle;

#[async_trait::async_trait]
impl RepoHandle for StubRepoHandle {
    async fn snapshot(&self) -> Result<RepoSnapshot> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }

    fn object_store(&self) -> &dyn ObjectStore {
        &StubObjectStore
    }

    fn refs_store(&self) -> &dyn RefsStore {
        &StubRefsStore
    }

    fn workdir(&self) -> &dyn Workdir {
        &StubWorkdir
    }

    fn index_reader(&self) -> &dyn IndexReader {
        &StubIndexReader
    }

    async fn diff_name_status(&self, _range: &str) -> Result<String> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }

    async fn diff_numstat(&self, _range: &str) -> Result<String> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }
}

/// Stub object store.
pub struct StubObjectStore;

#[async_trait::async_trait]
impl ObjectStore for StubObjectStore {
    async fn read_commit(&self, _id: &str) -> Result<Commit> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }

    async fn read_tree(&self, _id: &str) -> Result<Tree> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }

    async fn read_blob(&self, _id: &str) -> Result<Blob> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }
}

/// Stub refs store.
pub struct StubRefsStore;

#[async_trait::async_trait]
impl RefsStore for StubRefsStore {
    async fn all_refs(&self) -> Result<Vec<RefInfo>> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }

    async fn resolve_ref(&self, _name: &str) -> Result<String> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }
}

/// Stub workdir.
pub struct StubWorkdir;

#[async_trait::async_trait]
impl Workdir for StubWorkdir {
    async fn status(&self) -> Result<WorkdirStatus> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }
}

/// Stub index reader.
pub struct StubIndexReader;

#[async_trait::async_trait]
impl IndexReader for StubIndexReader {
    async fn staged_entries(&self) -> Result<Vec<IndexEntry>> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Git backend not implemented",
        ))
    }
}
