//! Request DTOs for the repo-lens API.

use crate::bounds::{Cursor, MaxBytes, MaxHunks, WindowSize};
use crate::paging::Paging;
use serde::{Deserialize, Serialize};

/// Top-level request envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// API version
    pub version: crate::ApiVersion,
    /// Request ID for correlation
    pub id: String,
    /// The actual request payload
    pub payload: RequestPayload,
}

/// Request payload variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestPayload {
    /// Get repository status
    Status(StatusRequest),
    /// Get commit log
    Log(LogRequest),
    /// Get commit graph window
    Graph(GraphRequest),
    /// Get commit details
    ShowCommit(ShowCommitRequest),
    /// Get diff summary
    DiffSummary(DiffSummaryRequest),
    /// Get diff content
    DiffContent(DiffContentRequest),
    /// Get blame information
    Blame(BlameRequest),
    /// Get branch list
    Branches(BranchesRequest),
    /// Get tag list
    Tags(TagsRequest),
    /// Get remote list
    Remotes(RemotesRequest),
    /// Checkout operation
    Checkout(CheckoutRequest),
    /// Commit operation
    Commit(CommitRequest),
    /// Fetch operation
    Fetch(FetchRequest),
    /// Push operation
    Push(PushRequest),
    /// Merge operation
    Merge(MergeRequest),
    /// Rebase operation
    Rebase(RebaseRequest),
    /// Stash operation
    Stash(StashRequest),
    /// Watch for events
    Watch(WatchRequest),
}

// Query requests

/// Status request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusRequest {
    /// Repository path
    pub repo_path: String,
}

/// Log request with pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRequest {
    /// Repository path
    pub repo_path: String,
    /// Pagination parameters
    #[serde(flatten)]
    pub paging: Paging,
    /// Optional revision range
    pub revision_range: Option<String>,
}

/// Graph request for commit graph window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRequest {
    /// Repository path
    pub repo_path: String,
    /// Window size for the graph
    pub window_size: WindowSize,
    /// Cursor for resuming
    pub cursor: Cursor,
    /// Optional revision range
    pub revision_range: Option<String>,
}

/// Show commit request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowCommitRequest {
    /// Repository path
    pub repo_path: String,
    /// Commit OID
    pub commit_id: String,
}

/// Diff summary request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummaryRequest {
    /// Repository path
    pub repo_path: String,
    /// From revision (optional for working directory)
    pub from: Option<String>,
    /// To revision (optional for working directory)
    pub to: Option<String>,
    /// Maximum bytes to process
    pub max_bytes: MaxBytes,
    /// Maximum hunks to return
    pub max_hunks: MaxHunks,
}

/// Diff content request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffContentRequest {
    /// Repository path
    pub repo_path: String,
    /// From revision (optional for working directory)
    pub from: Option<String>,
    /// To revision (optional for working directory)
    pub to: Option<String>,
    /// Optional path filter
    pub path: Option<String>,
    /// Maximum bytes to return
    pub max_bytes: MaxBytes,
}

/// Blame request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameRequest {
    /// Repository path
    pub repo_path: String,
    /// File path
    pub path: String,
    /// Optional revision
    pub revision: Option<String>,
}

/// Branches request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchesRequest {
    /// Repository path
    pub repo_path: String,
}

/// Tags request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsRequest {
    /// Repository path
    pub repo_path: String,
}

/// Remotes request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotesRequest {
    /// Repository path
    pub repo_path: String,
}

// Mutation requests

/// Checkout request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutRequest {
    /// Repository path
    pub repo_path: String,
    /// Target to checkout
    pub target: String,
    /// Create new branch
    pub create_branch: bool,
}

/// Commit request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    /// Repository path
    pub repo_path: String,
    /// Commit message
    pub message: String,
    /// Author name
    pub author_name: Option<String>,
    /// Author email
    pub author_email: Option<String>,
}

/// Fetch request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRequest {
    /// Repository path
    pub repo_path: String,
    /// Remote name (default: origin)
    pub remote: Option<String>,
    /// Refspecs to fetch
    pub refspecs: Option<Vec<String>>,
}

/// Push request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    /// Repository path
    pub repo_path: String,
    /// Remote name (default: origin)
    pub remote: Option<String>,
    /// Refspecs to push
    pub refspecs: Option<Vec<String>>,
    /// Force push
    pub force: bool,
}

/// Merge request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequest {
    /// Repository path
    pub repo_path: String,
    /// Branch/commit to merge
    pub source: String,
    /// Commit message
    pub message: Option<String>,
}

/// Rebase request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebaseRequest {
    /// Repository path
    pub repo_path: String,
    /// Branch/commit to rebase onto
    pub onto: String,
    /// Upstream branch (optional)
    pub upstream: Option<String>,
}

/// Stash request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashRequest {
    /// Repository path
    pub repo_path: String,
    /// Stash message
    pub message: Option<String>,
}

/// Watch request for event stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchRequest {
    /// Repository path
    pub repo_path: String,
}
