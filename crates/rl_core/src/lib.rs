//! Pure repo engine for repo-lens with query planning and caching coordination.
//!
//! This crate provides the core engine logic that coordinates Git operations,
//! caching, and query execution without any CLI/IPC/UI dependencies.

use rl_api::{response::ResponsePayload, Error, Request, Response};
use rl_git::CliBackend;
use rl_index::IndexManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::Instrument;

pub mod telemetry;

#[allow(dead_code)]
#[async_trait::async_trait]
trait Handler {
    type Request;
    type Response;

    async fn handle_impl(&self, request: Self::Request) -> Result<Self::Response, Error>;
}

/// Long-lived engine instance managing a repository.
pub struct RepoEngine {
    /// Engine configuration
    #[allow(dead_code)]
    config: EngineConfig,
    /// Git backend
    #[allow(dead_code)]
    git_backend: Box<dyn rl_git::GitBackend>,
    /// Index manager for caching
    #[allow(dead_code)]
    index_manager: IndexManager,
    /// Scheduler for query execution
    #[allow(dead_code)]
    scheduler: Scheduler,
}

fn parse_diff_summary(
    name_status: &str,
    numstat: &str,
) -> Result<rl_api::response::DiffSummary, Error> {
    use rl_api::response::{ChangeType, FileChange};
    use std::collections::HashMap;

    let mut changes = Vec::new();
    let mut numstat_map: HashMap<String, (usize, usize)> = HashMap::new();

    for line in numstat.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let added = parts[0].parse().unwrap_or(0);
            let deleted = parts[1].parse().unwrap_or(0);
            let path = parts[2..].join(" ");
            numstat_map.insert(path, (added, deleted));
        }
    }

    for line in name_status.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() {
            continue;
        }

        let status_code = parts[0].chars().next().unwrap_or(' ');
        let (change_type, path, old_path) = match status_code {
            'A' => {
                if parts.len() < 2 {
                    continue;
                }
                (ChangeType::Added, parts[1].to_string(), None)
            }
            'M' => {
                if parts.len() < 2 {
                    continue;
                }
                (ChangeType::Modified, parts[1].to_string(), None)
            }
            'D' => {
                if parts.len() < 2 {
                    continue;
                }
                (ChangeType::Deleted, parts[1].to_string(), None)
            }
            'R' => {
                if parts.len() < 3 {
                    continue;
                }
                (
                    ChangeType::Renamed,
                    parts[2].to_string(),
                    Some(parts[1].to_string()),
                )
            }
            _ => continue,
        };

        let (additions, deletions) = numstat_map.get(&path).copied().unwrap_or((0, 0));

        changes.push(FileChange {
            path,
            change_type,
            additions,
            deletions,
            old_path,
        });
    }

    let files_changed = changes.len();
    let additions = changes.iter().map(|c| c.additions).sum();
    let deletions = changes.iter().map(|c| c.deletions).sum();

    Ok(rl_api::response::DiffSummary {
        files_changed,
        additions,
        deletions,
        changes,
    })
}

#[allow(clippy::new_without_default)]
impl RepoEngine {
    /// Create a new engine with default configuration.
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
            git_backend: Box::new(CliBackend::new()),
            index_manager: IndexManager::new(),
            scheduler: Scheduler::new(),
        }
    }

    /// Create a new engine with custom configuration.
    pub fn with_config(config: EngineConfig) -> Self {
        Self {
            config,
            git_backend: Box::new(CliBackend::new()),
            index_manager: IndexManager::new(),
            scheduler: Scheduler::new(),
        }
    }

    /// Handle a request and return a response.
    pub async fn handle(&self, request: Request) -> Response {
        let request_id = telemetry::new_request_id();
        let request_type = format!("{:?}", request.payload);

        // Extract repo path from request
        let repo_path = extract_repo_path(&request.payload);

        let span = telemetry::RequestSpan::new(&request_id, &repo_path, &request_type);

        let result = async {
            tracing::info!("handling request");

            let result = match request.payload {
                rl_api::request::RequestPayload::Status(req) => {
                    step!("status", { self.handle_status(req).await })
                }
                rl_api::request::RequestPayload::Log(req) => {
                    step!("log", { self.handle_log(req).await })
                }
                rl_api::request::RequestPayload::Graph(req) => {
                    step!("graph", { self.handle_graph(req).await })
                }
                rl_api::request::RequestPayload::ShowCommit(req) => {
                    step!("show_commit", { self.handle_show_commit(req).await })
                }
                rl_api::request::RequestPayload::DiffSummary(req) => {
                    step!("diff_summary", { self.handle_diff_summary(req).await })
                }
                rl_api::request::RequestPayload::DiffContent(req) => {
                    step!("diff_content", { self.handle_diff_content(req).await })
                }
                rl_api::request::RequestPayload::Blame(req) => {
                    step!("blame", { self.handle_blame(req).await })
                }
                rl_api::request::RequestPayload::Branches(req) => {
                    step!("branches", { self.handle_branches(req).await })
                }
                rl_api::request::RequestPayload::Tags(req) => {
                    step!("tags", { self.handle_tags(req).await })
                }
                rl_api::request::RequestPayload::Remotes(req) => {
                    step!("remotes", { self.handle_remotes(req).await })
                }
                rl_api::request::RequestPayload::Checkout(req) => {
                    step!("checkout", { self.handle_checkout(req).await })
                }
                rl_api::request::RequestPayload::Commit(req) => {
                    step!("commit", { self.handle_commit(req).await })
                }
                rl_api::request::RequestPayload::Fetch(req) => {
                    step!("fetch", { self.handle_fetch(req).await })
                }
                rl_api::request::RequestPayload::Push(req) => {
                    step!("push", { self.handle_push(req).await })
                }
                rl_api::request::RequestPayload::Merge(req) => {
                    step!("merge", { self.handle_merge(req).await })
                }
                rl_api::request::RequestPayload::Rebase(req) => {
                    step!("rebase", { self.handle_rebase(req).await })
                }
                rl_api::request::RequestPayload::Stash(req) => {
                    step!("stash", { self.handle_stash(req).await })
                }
                rl_api::request::RequestPayload::Watch(req) => {
                    step!("watch", { self.handle_watch(req).await })
                }
            };

            match &result {
                Ok(_) => tracing::info!("request completed successfully"),
                Err(e) => tracing::error!(error = %e, "request failed"),
            }

            result
        }
        .instrument(span.enter())
        .await;

        Response {
            id: request.id,
            result,
        }
    }

    // Handler implementations

    async fn handle_status(
        &self,
        req: rl_api::request::StatusRequest,
    ) -> Result<ResponsePayload, Error> {
        use std::path::Path;

        let repo_path = Path::new(&req.repo_path);

        // Step 1: Open the repository
        let repo_handle = step!("git_open_repo", { self.git_backend.open_repo(repo_path).await })?;

        // Step 2: Get repository snapshot (HEAD, branch)
        let snapshot = step!("git_snapshot", { repo_handle.snapshot().await })?;

        // Step 3: Get working directory status (runs git status --porcelain=v1)
        let workdir_status =
            step!("git_status_porcelain", { repo_handle.workdir().status().await })?;

        // Step 4: Build response
        let response = step!("build_response", {
            // Determine which files are staged by looking at the index status
            // For now, we'll derive this from the workdir status
            // Files with index changes (XY where X != ' ') are staged
            let staged = workdir_status.added.clone();

            Ok(ResponsePayload::Status(rl_api::response::StatusView {
                branch: snapshot.branch,
                head: snapshot.head,
                workdir: rl_api::response::WorkdirStatus {
                    modified: workdir_status.modified.clone(),
                    added: Vec::new(), // Files only in workdir, not staged
                    deleted: workdir_status.deleted.clone(),
                    renamed: workdir_status.renamed.clone(),
                    untracked: workdir_status.untracked.clone(),
                },
                index: rl_api::response::IndexStatus { staged },
            }))
        })?;

        Ok(response)
    }

    async fn handle_log(
        &self,
        _req: rl_api::request::LogRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Log not implemented",
        ))
    }

    async fn handle_graph(
        &self,
        _req: rl_api::request::GraphRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Graph not implemented",
        ))
    }

    async fn handle_show_commit(
        &self,
        _req: rl_api::request::ShowCommitRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Show commit not implemented",
        ))
    }

    async fn handle_diff_summary(
        &self,
        req: rl_api::request::DiffSummaryRequest,
    ) -> Result<ResponsePayload, Error> {
        use std::path::Path;

        let repo_path = Path::new(&req.repo_path);

        let repo_handle = step!("git_open_repo", { self.git_backend.open_repo(repo_path).await })?;

        let from = req.from.as_deref().unwrap_or("HEAD");
        let to = req.to.as_deref().unwrap_or("");
        let range = if to.is_empty() {
            from.to_string()
        } else {
            format!("{}..{}", from, to)
        };

        let name_status_output =
            step!("git_diff_name_status", { repo_handle.diff_name_status(&range).await })?;

        let numstat_output =
            step!("git_diff_numstat", { repo_handle.diff_numstat(&range).await })?;

        let response = step!("parse_diff", {
            parse_diff_summary(&name_status_output, &numstat_output)
        })?;

        Ok(ResponsePayload::DiffSummary(response))
    }

    async fn handle_diff_content(
        &self,
        _req: rl_api::request::DiffContentRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Diff content not implemented",
        ))
    }

    async fn handle_blame(
        &self,
        _req: rl_api::request::BlameRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Blame not implemented",
        ))
    }

    async fn handle_branches(
        &self,
        _req: rl_api::request::BranchesRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Branches not implemented",
        ))
    }

    async fn handle_tags(
        &self,
        _req: rl_api::request::TagsRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Tags not implemented",
        ))
    }

    async fn handle_remotes(
        &self,
        _req: rl_api::request::RemotesRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Remotes not implemented",
        ))
    }

    async fn handle_checkout(
        &self,
        _req: rl_api::request::CheckoutRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Checkout not implemented",
        ))
    }

    async fn handle_commit(
        &self,
        _req: rl_api::request::CommitRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Commit not implemented",
        ))
    }

    async fn handle_fetch(
        &self,
        _req: rl_api::request::FetchRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Fetch not implemented",
        ))
    }

    async fn handle_push(
        &self,
        _req: rl_api::request::PushRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Push not implemented",
        ))
    }

    async fn handle_merge(
        &self,
        _req: rl_api::request::MergeRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Merge not implemented",
        ))
    }

    async fn handle_rebase(
        &self,
        _req: rl_api::request::RebaseRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Rebase not implemented",
        ))
    }

    async fn handle_stash(
        &self,
        _req: rl_api::request::StashRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Stash not implemented",
        ))
    }

    async fn handle_watch(
        &self,
        _req: rl_api::request::WatchRequest,
    ) -> Result<ResponsePayload, Error> {
        Err(Error::new(
            rl_api::ErrorCode::GitBackendError,
            "Watch not implemented",
        ))
    }
}

/// Engine configuration.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum concurrent queries
    pub max_concurrent_queries: usize,
    /// Query timeout in milliseconds
    pub query_timeout_ms: u64,
    /// Cache configuration
    pub cache_enabled: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_queries: 10,
            query_timeout_ms: 30000, // 30 seconds
            cache_enabled: true,
        }
    }
}

/// Simple cancellation token.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    /// Internal cancellation state
    cancelled: Arc<RwLock<bool>>,
}

impl CancellationToken {
    /// Create a new cancellation token.
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(RwLock::new(false)),
        }
    }

    /// Check if the operation has been cancelled.
    pub async fn is_cancelled(&self) -> bool {
        *self.cancelled.read().await
    }

    /// Cancel the operation.
    pub async fn cancel(&self) {
        *self.cancelled.write().await = true;
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Query scheduler with priority queues.
pub struct Scheduler {
    /// UI immediate priority queue
    ui_immediate: Vec<PendingQuery>,
    /// UI prefetch priority queue
    ui_prefetch: Vec<PendingQuery>,
    /// Maintenance priority queue
    maintenance: Vec<PendingQuery>,
}

#[allow(clippy::new_without_default)]
impl Scheduler {
    /// Create a new scheduler.
    pub fn new() -> Self {
        Self {
            ui_immediate: Vec::new(),
            ui_prefetch: Vec::new(),
            maintenance: Vec::new(),
        }
    }

    /// Schedule a query with the given priority.
    pub fn schedule(&mut self, query: PendingQuery, priority: Priority) {
        match priority {
            Priority::UiImmediate => self.ui_immediate.push(query),
            Priority::UiPrefetch => self.ui_prefetch.push(query),
            Priority::Maintenance => self.maintenance.push(query),
        }
    }

    /// Get the next query to execute.
    pub fn next_query(&mut self) -> Option<PendingQuery> {
        // UI immediate takes precedence
        if let Some(query) = self.ui_immediate.pop() {
            return Some(query);
        }
        // Then UI prefetch
        if let Some(query) = self.ui_prefetch.pop() {
            return Some(query);
        }
        // Finally maintenance
        self.maintenance.pop()
    }
}

/// Pending query in the scheduler.
#[derive(Debug)]
pub struct PendingQuery {
    /// Query ID
    pub id: String,
    /// Query payload
    pub payload: rl_api::request::RequestPayload,
    /// Cancellation token
    pub cancellation: CancellationToken,
}

/// Query execution priority.
#[derive(Debug, Clone, Copy)]
pub enum Priority {
    /// Immediate UI response required
    UiImmediate,
    /// UI prefetch (can be cancelled by immediate)
    UiPrefetch,
    /// Background maintenance work
    Maintenance,
}

/// Extract repo path from request payload for telemetry.
fn extract_repo_path(payload: &rl_api::request::RequestPayload) -> String {
    use rl_api::request::RequestPayload;

    match payload {
        RequestPayload::Status(req) => req.repo_path.clone(),
        RequestPayload::Log(req) => req.repo_path.clone(),
        RequestPayload::Graph(req) => req.repo_path.clone(),
        RequestPayload::ShowCommit(req) => req.repo_path.clone(),
        RequestPayload::DiffSummary(req) => req.repo_path.clone(),
        RequestPayload::DiffContent(req) => req.repo_path.clone(),
        RequestPayload::Blame(req) => req.repo_path.clone(),
        RequestPayload::Branches(req) => req.repo_path.clone(),
        RequestPayload::Tags(req) => req.repo_path.clone(),
        RequestPayload::Remotes(req) => req.repo_path.clone(),
        RequestPayload::Checkout(req) => req.repo_path.clone(),
        RequestPayload::Commit(req) => req.repo_path.clone(),
        RequestPayload::Fetch(req) => req.repo_path.clone(),
        RequestPayload::Push(req) => req.repo_path.clone(),
        RequestPayload::Merge(req) => req.repo_path.clone(),
        RequestPayload::Rebase(req) => req.repo_path.clone(),
        RequestPayload::Stash(req) => req.repo_path.clone(),
        RequestPayload::Watch(req) => req.repo_path.clone(),
    }
}
