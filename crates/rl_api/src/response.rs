//! Response DTOs for the repo-lens API.

use crate::bounds::Cursor;
use crate::paging::StreamingChunk;
use serde::{Deserialize, Serialize};

/// Top-level response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Request ID for correlation
    pub id: String,
    /// Response payload or error
    #[serde(flatten)]
    pub result: Result<ResponsePayload, crate::Error>,
}

/// Response payload variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponsePayload {
    /// Status response
    Status(StatusView),
    /// Log response
    Log(CommitListPage),
    /// Graph response
    Graph(CommitGraphWindow),
    /// Show commit response
    ShowCommit(CommitDetails),
    /// Diff summary response
    DiffSummary(DiffSummary),
    /// Diff content response (streaming)
    DiffContent(StreamingChunk<DiffChunk>),
    /// Blame response (streaming)
    Blame(StreamingChunk<BlameChunk>),
    /// Branches response
    Branches(BranchList),
    /// Tags response
    Tags(TagList),
    /// Remotes response
    Remotes(RemoteList),
    /// Generic operation result
    OperationResult(OperationResult),
    /// Merge result
    MergeResult(MergeResult),
    /// Rebase result
    RebaseResult(RebaseResult),
    /// Progress stream
    Progress(StreamingChunk<ProgressUpdate>),
    /// Event stream
    Event(crate::Event),
}

// Data types

/// Repository status view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusView {
    /// Current branch name
    pub branch: Option<String>,
    /// HEAD commit OID
    pub head: Option<String>,
    /// Working directory status
    pub workdir: WorkdirStatus,
    /// Index status
    pub index: IndexStatus,
}

/// Working directory status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkdirStatus {
    /// Modified files
    pub modified: Vec<String>,
    /// Added files
    pub added: Vec<String>,
    /// Deleted files
    pub deleted: Vec<String>,
    /// Renamed files
    pub renamed: Vec<(String, String)>,
    /// Untracked files
    pub untracked: Vec<String>,
}

/// Index status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatus {
    /// Staged files
    pub staged: Vec<String>,
}

/// Paged commit list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitListPage {
    /// Commits in this page
    pub commits: Vec<CommitSummary>,
    /// Cursor for next page
    pub next_cursor: Option<Cursor>,
    /// Whether this is the final page
    pub has_more: bool,
}

/// Commit summary for lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSummary {
    /// Commit OID
    pub id: String,
    /// Short commit message
    pub message: String,
    /// Author name
    pub author_name: String,
    /// Author email
    pub author_email: String,
    /// Commit time (Unix timestamp)
    pub time: i64,
    /// Parent commit IDs
    pub parents: Vec<String>,
}

/// Commit graph window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitGraphWindow {
    /// Commits with graph information
    pub commits: Vec<CommitGraphNode>,
    /// Cursor for next page
    pub next_cursor: Option<Cursor>,
    /// Whether this is the final page
    pub has_more: bool,
}

/// Commit with graph lane information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitGraphNode {
    /// Commit summary
    #[serde(flatten)]
    pub commit: CommitSummary,
    /// Graph lanes for this commit
    pub lanes: Vec<GraphLane>,
}

/// Graph lane representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLane {
    /// Lane index
    pub index: usize,
    /// Lane type
    pub lane_type: LaneType,
}

/// Type of graph lane.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaneType {
    /// Commit on this lane
    Commit,
    /// Merge line
    Merge,
    /// Branch line
    Branch,
    /// Empty space
    Empty,
}

/// Detailed commit information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitDetails {
    /// Commit summary
    #[serde(flatten)]
    pub summary: CommitSummary,
    /// Full commit message
    pub full_message: String,
    /// Changed files summary
    pub changed_files: Vec<FileChange>,
}

/// File change in a commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// File path
    pub path: String,
    /// Change type
    pub change_type: ChangeType,
    /// Lines added
    pub additions: usize,
    /// Lines deleted
    pub deletions: usize,
    /// Old path (for renames)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
}

/// Type of file change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// File added
    Added,
    /// File modified
    Modified,
    /// File deleted
    Deleted,
    /// File renamed
    Renamed,
}

/// Diff summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    /// Total files changed
    pub files_changed: usize,
    /// Total additions
    pub additions: usize,
    /// Total deletions
    pub deletions: usize,
    /// File changes
    pub changes: Vec<FileChange>,
}

/// Chunk of diff content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChunk {
    /// File path
    pub path: String,
    /// Diff hunks in this chunk
    pub hunks: Vec<DiffHunk>,
}

/// Diff hunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    /// Old file range
    pub old_range: Range,
    /// New file range
    pub new_range: Range,
    /// Hunk header
    pub header: String,
    /// Lines in the hunk
    pub lines: Vec<DiffLine>,
}

/// Range in a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    /// Starting line number
    pub start: usize,
    /// Number of lines
    pub count: usize,
}

/// Line in a diff hunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    /// Line type
    pub line_type: DiffLineType,
    /// Line number in old file
    pub old_line: Option<usize>,
    /// Line number in new file
    pub new_line: Option<usize>,
    /// Line content
    pub content: String,
}

/// Type of diff line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffLineType {
    /// Context line
    Context,
    /// Added line
    Addition,
    /// Deleted line
    Deletion,
}

/// Chunk of blame information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameChunk {
    /// File path
    pub path: String,
    /// Blame lines in this chunk
    pub lines: Vec<BlameLine>,
}

/// Line in blame output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameLine {
    /// Line number
    pub line_number: usize,
    /// Commit ID
    pub commit_id: String,
    /// Author name
    pub author_name: String,
    /// Author email
    pub author_email: String,
    /// Line content
    pub content: String,
}

/// Branch list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchList {
    /// Local branches
    pub local: Vec<BranchInfo>,
    /// Remote branches
    pub remote: Vec<BranchInfo>,
    /// Current branch name
    pub current: Option<String>,
}

/// Branch information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    /// Commit OID
    pub commit_id: String,
    /// Whether this branch is remote
    pub is_remote: bool,
}

/// Tag list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagList {
    /// Tags
    pub tags: Vec<TagInfo>,
}

/// Tag information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    /// Tag name
    pub name: String,
    /// Commit OID
    pub commit_id: String,
    /// Tag message
    pub message: Option<String>,
}

/// Remote list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteList {
    /// Remotes
    pub remotes: Vec<RemoteInfo>,
}

/// Remote information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInfo {
    /// Remote name
    pub name: String,
    /// Remote URL
    pub url: String,
    /// Fetch refspecs
    pub fetch_refspecs: Vec<String>,
    /// Push refspecs
    pub push_refspecs: Vec<String>,
}

/// Generic operation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
}

/// Merge operation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// Whether the merge succeeded
    pub success: bool,
    /// Merge type
    pub merge_type: MergeType,
    /// Conflicts (if any)
    pub conflicts: Vec<String>,
}

/// Type of merge.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeType {
    /// Fast-forward merge
    FastForward,
    /// Merge commit created
    MergeCommit,
    /// Already up to date
    UpToDate,
}

/// Rebase operation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebaseResult {
    /// Whether the rebase succeeded
    pub success: bool,
    /// Number of commits rebased
    pub commits_rebased: usize,
    /// Conflicts (if any)
    pub conflicts: Vec<String>,
}

/// Progress update for long-running operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    /// Operation stage
    pub stage: String,
    /// Current progress (0-100)
    pub progress: u8,
    /// Optional message
    pub message: Option<String>,
}
