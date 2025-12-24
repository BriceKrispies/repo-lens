//! Git CLI backend implementation using std::process::Command.

use crate::{GitBackend, RepoHandle, RepoSnapshot, Result};
use std::path::Path;

/// Git CLI backend that shells out to the git command.
pub struct CliBackend;

impl CliBackend {
    /// Create a new CLI backend.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CliBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl GitBackend for CliBackend {
    async fn open_repo(&self, path: &Path) -> Result<Box<dyn RepoHandle>> {
        // Verify it's a git repository
        let is_valid = self.is_repo(path).await?;
        if !is_valid {
            return Err(rl_api::Error::new(
                rl_api::ErrorCode::RepoNotFound,
                format!("Not a git repository: {}", path.display()),
            ));
        }

        Ok(Box::new(CliRepoHandle::new(path)))
    }

    async fn is_repo(&self, path: &Path) -> Result<bool> {
        let output = tokio::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("rev-parse")
            .arg("--git-dir")
            .output()
            .await
            .map_err(|e| {
                rl_api::Error::new(
                    rl_api::ErrorCode::GitBackendError,
                    format!("Failed to execute git: {}", e),
                )
            })?;

        Ok(output.status.success())
    }
}

/// Repository handle using Git CLI.
pub struct CliRepoHandle {
    path: std::path::PathBuf,
    workdir: CliWorkdir,
}

impl CliRepoHandle {
    fn new(path: impl AsRef<Path>) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        Self {
            workdir: CliWorkdir {
                path: path_buf.clone(),
            },
            path: path_buf,
        }
    }

    async fn run_git(&self, args: &[&str]) -> Result<std::process::Output> {
        tokio::process::Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .args(args)
            .output()
            .await
            .map_err(|e| {
                rl_api::Error::new(
                    rl_api::ErrorCode::GitBackendError,
                    format!("Failed to execute git: {}", e),
                )
            })
    }
}

#[async_trait::async_trait]
impl RepoHandle for CliRepoHandle {
    async fn snapshot(&self) -> Result<RepoSnapshot> {
        // Get HEAD commit
        let head_output = self.run_git(&["rev-parse", "HEAD"]).await?;
        let head = if head_output.status.success() {
            Some(
                String::from_utf8_lossy(&head_output.stdout)
                    .trim()
                    .to_string(),
            )
        } else {
            None
        };

        // Get current branch
        let branch_output = self.run_git(&["rev-parse", "--abbrev-ref", "HEAD"]).await?;
        let branch = if branch_output.status.success() {
            let branch_name = String::from_utf8_lossy(&branch_output.stdout)
                .trim()
                .to_string();
            if branch_name != "HEAD" {
                Some(branch_name)
            } else {
                None
            }
        } else {
            None
        };

        Ok(RepoSnapshot {
            path: self.path.clone(),
            head,
            branch,
            refs: Vec::new(), // TODO: implement if needed
        })
    }

    fn object_store(&self) -> &dyn crate::ObjectStore {
        &CliObjectStore
    }

    fn refs_store(&self) -> &dyn crate::RefsStore {
        &CliRefsStore
    }

    fn workdir(&self) -> &dyn crate::Workdir {
        &self.workdir
    }

    fn index_reader(&self) -> &dyn crate::IndexReader {
        &CliIndexReader
    }

    async fn diff_name_status(&self, range: &str) -> Result<String> {
        let output = tokio::process::Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("diff")
            .arg("--name-status")
            .arg("-M")
            .arg(range)
            .output()
            .await
            .map_err(|e| {
                rl_api::Error::new(
                    rl_api::ErrorCode::GitBackendError,
                    format!("Failed to execute git diff: {}", e),
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(rl_api::Error::new(
                rl_api::ErrorCode::GitBackendError,
                format!("git diff failed: {}", stderr),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn diff_numstat(&self, range: &str) -> Result<String> {
        let output = tokio::process::Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("diff")
            .arg("--numstat")
            .arg(range)
            .output()
            .await
            .map_err(|e| {
                rl_api::Error::new(
                    rl_api::ErrorCode::GitBackendError,
                    format!("Failed to execute git diff: {}", e),
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(rl_api::Error::new(
                rl_api::ErrorCode::GitBackendError,
                format!("git diff failed: {}", stderr),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// CLI-based workdir implementation.
pub struct CliWorkdir {
    path: std::path::PathBuf,
}

#[async_trait::async_trait]
impl crate::Workdir for CliWorkdir {
    async fn status(&self) -> Result<crate::WorkdirStatus> {
        let output = tokio::process::Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("status")
            .arg("--porcelain=v1")
            .arg("-z") // Null-terminated for proper handling of special chars
            .output()
            .await
            .map_err(|e| {
                rl_api::Error::new(
                    rl_api::ErrorCode::GitBackendError,
                    format!("Failed to execute git status: {}", e),
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(rl_api::Error::new(
                rl_api::ErrorCode::GitBackendError,
                format!("git status failed: {}", stderr),
            ));
        }

        // Parse porcelain output
        parse_status_porcelain(&output.stdout)
    }
}

/// Parse git status --porcelain=v1 -z output.
///
/// Format: XY PATH
/// - X shows status in index (staged)
/// - Y shows status in working tree (unstaged)
///
/// Returns WorkdirStatus which contains all changes (both staged and unstaged).
/// The caller needs to separate them based on the XY codes.
fn parse_status_porcelain(output: &[u8]) -> Result<crate::WorkdirStatus> {
    let mut modified = Vec::new();
    let mut added = Vec::new();
    let mut deleted = Vec::new();
    let mut renamed = Vec::new();
    let mut untracked = Vec::new();

    // Split on null bytes
    let entries: Vec<&[u8]> = output
        .split(|&b| b == 0)
        .filter(|e| !e.is_empty())
        .collect();

    let mut i = 0;
    while i < entries.len() {
        let entry = entries[i];
        if entry.len() < 3 {
            i += 1;
            continue;
        }

        let x = entry[0]; // Index status
        let y = entry[1]; // Working tree status
        let path = String::from_utf8_lossy(&entry[3..]).to_string();

        // Parse status code (XY format)
        // X is index (staged), Y is working tree (unstaged)
        match (x, y) {
            (b'?', b'?') => {
                // Untracked
                untracked.push(path);
            }
            (b'A', _) => {
                // Added to index (staged)
                added.push(path);
            }
            (b'M', b' ') => {
                // Modified in index only (staged modification)
                modified.push(path);
            }
            (b' ', b'M') | (b'M', b'M') => {
                // Modified in working tree (unstaged)
                modified.push(path);
            }
            (b'D', _) | (b' ', b'D') => {
                // Deleted
                deleted.push(path);
            }
            (b'R', _) => {
                // Renamed - next entry is the old name
                if i + 1 < entries.len() {
                    let old_path = String::from_utf8_lossy(entries[i + 1]).to_string();
                    renamed.push((old_path, path));
                    i += 1; // Skip next entry
                }
            }
            _ => {
                // Handle any other cases by checking individual flags
                if x == b'M' || y == b'M' {
                    modified.push(path.clone());
                }
                if x == b'A' {
                    added.push(path.clone());
                }
                if x == b'D' || y == b'D' {
                    deleted.push(path.clone());
                }
            }
        }

        i += 1;
    }

    Ok(crate::WorkdirStatus {
        modified,
        added,
        deleted,
        renamed,
        untracked,
    })
}

// Stub implementations for other interfaces

struct CliObjectStore;

#[async_trait::async_trait]
impl crate::ObjectStore for CliObjectStore {
    async fn read_commit(&self, _id: &str) -> Result<crate::Commit> {
        Err(rl_api::Error::new(
            rl_api::ErrorCode::GitBackendError,
            "CLI object store not fully implemented",
        ))
    }

    async fn read_tree(&self, _id: &str) -> Result<crate::Tree> {
        Err(rl_api::Error::new(
            rl_api::ErrorCode::GitBackendError,
            "CLI object store not fully implemented",
        ))
    }

    async fn read_blob(&self, _id: &str) -> Result<crate::Blob> {
        Err(rl_api::Error::new(
            rl_api::ErrorCode::GitBackendError,
            "CLI object store not fully implemented",
        ))
    }
}

struct CliRefsStore;

#[async_trait::async_trait]
impl crate::RefsStore for CliRefsStore {
    async fn all_refs(&self) -> Result<Vec<crate::RefInfo>> {
        Err(rl_api::Error::new(
            rl_api::ErrorCode::GitBackendError,
            "CLI refs store not fully implemented",
        ))
    }

    async fn resolve_ref(&self, _name: &str) -> Result<String> {
        Err(rl_api::Error::new(
            rl_api::ErrorCode::GitBackendError,
            "CLI refs store not fully implemented",
        ))
    }
}

struct CliIndexReader;

#[async_trait::async_trait]
impl crate::IndexReader for CliIndexReader {
    async fn staged_entries(&self) -> Result<Vec<crate::IndexEntry>> {
        Err(rl_api::Error::new(
            rl_api::ErrorCode::GitBackendError,
            "CLI index reader not fully implemented",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_porcelain() {
        // Test basic untracked file
        let input = b"?? file.txt\0";
        let status = parse_status_porcelain(input).unwrap();
        assert_eq!(status.untracked, vec!["file.txt"]);
        assert!(status.modified.is_empty());
        assert!(status.added.is_empty());
        assert!(status.deleted.is_empty());

        // Test modified file
        let input = b" M modified.txt\0";
        let status = parse_status_porcelain(input).unwrap();
        assert_eq!(status.modified, vec!["modified.txt"]);
        assert!(status.untracked.is_empty());

        // Test added file
        let input = b"A  added.txt\0";
        let status = parse_status_porcelain(input).unwrap();
        assert_eq!(status.added, vec!["added.txt"]);

        // Test deleted file
        let input = b" D deleted.txt\0";
        let status = parse_status_porcelain(input).unwrap();
        assert_eq!(status.deleted, vec!["deleted.txt"]);

        // Test multiple files
        let input = b"?? untracked.txt\0 M modified.txt\0A  added.txt\0";
        let status = parse_status_porcelain(input).unwrap();
        assert_eq!(status.untracked, vec!["untracked.txt"]);
        assert_eq!(status.modified, vec!["modified.txt"]);
        assert_eq!(status.added, vec!["added.txt"]);
    }
}
