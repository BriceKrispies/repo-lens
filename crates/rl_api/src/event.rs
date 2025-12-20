//! Event DTOs for the repo-lens API.

use serde::{Deserialize, Serialize};

/// Event types for reactive UI updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    /// HEAD reference changed
    HeadChanged(HeadChangedEvent),
    /// Index changed
    IndexChanged(IndexChangedEvent),
    /// Working directory changed
    WorkdirChanged(WorkdirChangedEvent),
    /// References changed
    RefsChanged(RefsChangedEvent),
    /// Repository opened
    RepoOpened(RepoOpenedEvent),
    /// Repository closed
    RepoClosed(RepoClosedEvent),
    /// Operation progress update
    OperationProgress(OperationProgressEvent),
}

/// HEAD changed event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadChangedEvent {
    /// Repository path
    pub repo_path: String,
    /// New HEAD commit ID
    pub new_head: Option<String>,
    /// Previous HEAD commit ID
    pub old_head: Option<String>,
}

/// Index changed event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexChangedEvent {
    /// Repository path
    pub repo_path: String,
    /// Changed files
    pub changed_files: Vec<String>,
}

/// Working directory changed event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkdirChangedEvent {
    /// Repository path
    pub repo_path: String,
    /// Changed files
    pub changed_files: Vec<String>,
}

/// References changed event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefsChangedEvent {
    /// Repository path
    pub repo_path: String,
    /// Changed references
    pub changed_refs: Vec<String>,
}

/// Repository opened event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoOpenedEvent {
    /// Repository path
    pub repo_path: String,
}

/// Repository closed event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoClosedEvent {
    /// Repository path
    pub repo_path: String,
}

/// Operation progress event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProgressEvent {
    /// Repository path
    pub repo_path: String,
    /// Operation ID
    pub operation_id: String,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Progress message
    pub message: Option<String>,
}
