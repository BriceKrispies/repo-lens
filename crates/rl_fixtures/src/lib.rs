//! Test-only helpers for generating repos and edge cases.
//!
//! This crate provides utilities for creating synthetic Git repositories
//! with various edge cases (merges, renames, conflicts, large files)
//! for testing purposes.

pub mod synth_repo;

/// Repository generator for creating synthetic test repositories.
pub struct RepoGenerator {
    /// Repository configuration
    #[allow(dead_code)]
    config: RepoConfig,
}

impl RepoGenerator {
    /// Create a new repository generator.
    pub fn new() -> Self {
        Self {
            config: RepoConfig::default(),
        }
    }

    /// Create a new repository generator with custom config.
    pub fn with_config(config: RepoConfig) -> Self {
        Self { config }
    }

    /// Generate a basic repository with a single commit.
    pub fn generate_basic(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Stub implementation
        Err("Repository generation not implemented".into())
    }

    /// Generate a repository with merge commits.
    pub fn generate_with_merges(
        &self,
        _num_merges: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Stub implementation
        Err("Repository generation not implemented".into())
    }

    /// Generate a repository with renamed files.
    pub fn generate_with_renames(
        &self,
        _num_renames: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Stub implementation
        Err("Repository generation not implemented".into())
    }

    /// Generate a repository with conflicts.
    pub fn generate_with_conflicts(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Stub implementation
        Err("Repository generation not implemented".into())
    }

    /// Generate a repository with large files.
    pub fn generate_with_large_files(
        &self,
        _num_large_files: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Stub implementation
        Err("Repository generation not implemented".into())
    }
}

impl Default for RepoGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for repository generation.
#[derive(Debug, Clone)]
pub struct RepoConfig {
    /// Number of initial commits
    pub initial_commits: usize,
    /// Maximum file size in bytes
    pub max_file_size: usize,
    /// Number of branches to create
    pub num_branches: usize,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            initial_commits: 10,
            max_file_size: 1024 * 1024, // 1MB
            num_branches: 3,
        }
    }
}

/// File generator for creating test files with various characteristics.
pub struct FileGenerator;

impl FileGenerator {
    /// Generate a text file with the specified size.
    pub fn generate_text_file(_size_bytes: usize) -> String {
        // Stub implementation
        "Generated test content".to_string()
    }

    /// Generate a binary file with the specified size.
    pub fn generate_binary_file(_size_bytes: usize) -> Vec<u8> {
        // Stub implementation
        vec![0, 1, 2, 3, 4]
    }

    /// Generate a file that will cause merge conflicts.
    pub fn generate_conflict_file() -> String {
        // Stub implementation
        "line 1\nline 2\nline 3\n".to_string()
    }
}

/// Edge case repository templates.
pub enum RepoTemplate {
    /// Empty repository
    Empty,
    /// Single commit with one file
    SingleCommit,
    /// Linear history
    LinearHistory,
    /// History with merges
    MergeHistory,
    /// History with renames
    RenameHistory,
    /// History with conflicts
    ConflictHistory,
    /// Large repository
    LargeRepo,
}

impl RepoTemplate {
    /// Get the configuration for this template.
    pub fn config(&self) -> RepoConfig {
        match self {
            Self::Empty => RepoConfig {
                initial_commits: 0,
                ..Default::default()
            },
            Self::SingleCommit => RepoConfig {
                initial_commits: 1,
                ..Default::default()
            },
            Self::LinearHistory => RepoConfig {
                initial_commits: 100,
                num_branches: 1,
                ..Default::default()
            },
            Self::MergeHistory => RepoConfig {
                initial_commits: 50,
                num_branches: 5,
                ..Default::default()
            },
            Self::RenameHistory => RepoConfig {
                initial_commits: 20,
                num_branches: 2,
                ..Default::default()
            },
            Self::ConflictHistory => RepoConfig {
                initial_commits: 10,
                num_branches: 2,
                ..Default::default()
            },
            Self::LargeRepo => RepoConfig {
                initial_commits: 1000,
                max_file_size: 10 * 1024 * 1024, // 10MB
                num_branches: 10,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_generator_creation() {
        let generator = RepoGenerator::new();
        assert_eq!(generator.config.initial_commits, 10);
    }

    #[test]
    fn test_template_configs() {
        let empty_config = RepoTemplate::Empty.config();
        assert_eq!(empty_config.initial_commits, 0);

        let large_config = RepoTemplate::LargeRepo.config();
        assert_eq!(large_config.initial_commits, 1000);
        assert_eq!(large_config.num_branches, 10);
    }

    #[test]
    fn test_file_generation() {
        let text = FileGenerator::generate_text_file(100);
        assert!(!text.is_empty());

        let binary = FileGenerator::generate_binary_file(100);
        assert!(!binary.is_empty());
    }
}
