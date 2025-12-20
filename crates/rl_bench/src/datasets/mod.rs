//! Benchmark dataset management.
//!
//! This module handles downloading, caching, and preparing external Git repositories
//! used for performance benchmarking.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Dataset configuration from manifest.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// Unique dataset name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Git repository URL
    pub url: String,
    /// Pinned revision (tag, branch, or SHA)
    pub revision: String,
    /// Size category for informational purposes
    pub size_category: String,
}

/// Dataset manifest containing all available datasets
#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetManifest {
    pub datasets: Vec<Dataset>,
}

impl DatasetManifest {
    /// Load the dataset manifest from the embedded manifest.toml
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let manifest_content = include_str!("manifest.toml");
        let manifest: DatasetManifest = toml::from_str(manifest_content)?;
        Ok(manifest)
    }

    /// Find a dataset by name
    pub fn find_by_name(&self, name: &str) -> Option<&Dataset> {
        self.datasets.iter().find(|d| d.name == name)
    }

    /// Get all available dataset names
    #[allow(dead_code)]
    pub fn names(&self) -> Vec<String> {
        self.datasets.iter().map(|d| d.name.clone()).collect()
    }
}

/// Dataset resolver that handles cloning and checkout
pub struct DatasetResolver {
    /// Base directory for cached datasets
    cache_dir: PathBuf,
}

impl DatasetResolver {
    /// Create a new resolver with default cache directory
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let cache_dir = PathBuf::from("target/rl_bench/datasets");
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// Create a resolver with custom cache directory
    #[allow(dead_code)]
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        fs::create_dir_all(&cache_dir)?;
        Ok(Self { cache_dir })
    }

    /// Resolve a dataset by name, cloning if necessary and ensuring correct revision
    #[allow(dead_code)]
    pub fn resolve(&self, dataset: &Dataset) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dataset_path = self.cache_dir.join(&dataset.name);

        // Clone if doesn't exist
        if !dataset_path.exists() {
            println!("Cloning dataset '{}' from {}...", dataset.name, dataset.url);
            self.clone_repository(&dataset.url, &dataset_path)?;
        }

        // Ensure correct revision is checked out
        self.checkout_revision(&dataset_path, &dataset.revision)?;

        Ok(dataset_path)
    }

    /// Clone a repository to the specified path
    #[allow(dead_code)]
    fn clone_repository(&self, url: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let status = Command::new("git")
            .args(["clone", "--quiet", url, &path.to_string_lossy()])
            .status()?;

        if !status.success() {
            return Err(format!("Failed to clone repository from {}", url).into());
        }

        Ok(())
    }

    /// Checkout the specified revision in the repository
    #[allow(dead_code)]
    fn checkout_revision(
        &self,
        path: &Path,
        revision: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch latest changes
        let fetch_status = Command::new("git")
            .args(["fetch", "--quiet", "--tags"])
            .current_dir(path)
            .status()?;

        if !fetch_status.success() {
            return Err("Failed to fetch repository updates".into());
        }

        // Checkout the specific revision
        let checkout_status = Command::new("git")
            .args(["checkout", "--quiet", revision])
            .current_dir(path)
            .status()?;

        if !checkout_status.success() {
            return Err(format!("Failed to checkout revision {}", revision).into());
        }

        Ok(())
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// List all cached datasets
    pub fn list_cached(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut cached = Vec::new();

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        cached.push(name.to_string());
                    }
                }
            }
        }

        Ok(cached)
    }

    /// Remove a cached dataset
    #[allow(dead_code)]
    pub fn remove_cached(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.cache_dir.join(name);
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }
}

/// Get the default dataset (Git repository)
#[allow(dead_code)]
pub fn default_dataset() -> Result<Dataset, Box<dyn std::error::Error>> {
    let manifest = DatasetManifest::load()?;
    manifest
        .find_by_name("git")
        .cloned()
        .ok_or_else(|| "Default dataset 'git' not found in manifest".into())
}
