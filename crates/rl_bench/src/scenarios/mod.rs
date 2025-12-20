//! Benchmark scenarios for repo-lens.
//!
//! This module defines deterministic benchmark scenarios that correspond to
//! typical UI interactions, using pinned commits from real repositories.

use rl_api::{request::*, ApiVersion, Request};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A benchmark scenario with deterministic inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkScenario {
    /// Scenario name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// The request to execute
    pub request: Request,
}

/// Results from running a benchmark scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Scenario name
    pub scenario: String,
    /// Wall-clock time in nanoseconds
    pub wall_time_ns: u64,
    /// Whether the operation succeeded (even if "not implemented")
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Sentinel benchmark result with detailed timing and dataset info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelResult {
    /// Dataset information
    pub dataset: DatasetInfo,
    /// Scenario name
    pub scenario: String,
    /// Timing information
    pub timings: TimingInfo,
    /// Status
    pub status: String,
    /// Reason for status (null if pass)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Dataset information for benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    /// Dataset name
    pub name: String,
    /// Repository URL
    pub url: String,
    /// Revision (tag/commit)
    pub rev: String,
    /// Local path
    pub path: String,
    /// Whether the dataset exists locally
    pub exists: bool,
}

/// Timing information for benchmark runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingInfo {
    /// Cold run time in milliseconds (first execution)
    pub cold_ms: f64,
    /// Total warm run time in milliseconds (all iterations)
    pub warm_total_ms: f64,
    /// Average warm run time in milliseconds per iteration
    pub warm_avg_ms: f64,
    /// Number of warm iterations
    pub iterations: usize,
}

/// Collection of benchmark results from a run
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkRun {
    /// Timestamp of the run
    pub timestamp: String,
    /// Dataset used
    pub dataset: String,
    /// Results for each scenario
    pub results: Vec<BenchmarkResult>,
}

/// Generate benchmark scenarios for a given repository path
pub fn generate_scenarios(repo_path: &Path) -> Vec<BenchmarkScenario> {
    let repo_path_str = repo_path.to_string_lossy().to_string();

    vec![
        BenchmarkScenario {
            name: "engine_overhead".to_string(),
            description: "Measure engine overhead with minimal Status request".to_string(),
            request: Request {
                version: ApiVersion::V0,
                id: "bench-engine-overhead".to_string(),
                payload: RequestPayload::Status(StatusRequest {
                    repo_path: repo_path_str.clone(),
                }),
            },
        },
        BenchmarkScenario {
            name: "status".to_string(),
            description: "Get repository status".to_string(),
            request: Request {
                version: ApiVersion::V0,
                id: "bench-status".to_string(),
                payload: RequestPayload::Status(StatusRequest {
                    repo_path: repo_path_str.clone(),
                }),
            },
        },
        BenchmarkScenario {
            name: "log_page".to_string(),
            description: "Get commit log with pagination (200 commits)".to_string(),
            request: Request {
                version: ApiVersion::V0,
                id: "bench-log".to_string(),
                payload: RequestPayload::Log(LogRequest {
                    repo_path: repo_path_str.clone(),
                    paging: rl_api::Paging {
                        page_size: rl_api::PageSize::try_from(200).unwrap(),
                        cursor: rl_api::Cursor::initial(),
                    },
                    revision_range: None,
                }),
            },
        },
        BenchmarkScenario {
            name: "diff_summary".to_string(),
            description: "Get diff summary between two specific commits".to_string(),
            request: Request {
                version: ApiVersion::V0,
                id: "bench-diff-summary".to_string(),
                payload: RequestPayload::DiffSummary(DiffSummaryRequest {
                    repo_path: repo_path_str.clone(),
                    // Using commits that exist in Git v2.45.0
                    from: Some("HEAD~10".to_string()),
                    to: Some("HEAD".to_string()),
                    max_bytes: rl_api::MaxBytes::try_from(1024 * 1024).unwrap(),
                    max_hunks: rl_api::MaxHunks::try_from(1000).unwrap(),
                }),
            },
        },
    ]
}

/// Get the names of all available scenarios
#[allow(dead_code)]
pub fn scenario_names() -> Vec<String> {
    vec![
        "engine_overhead".to_string(),
        "status".to_string(),
        "log_page".to_string(),
        "diff_summary".to_string(),
    ]
}

/// Find a scenario by name
#[allow(dead_code)]
pub fn find_scenario<'a>(
    scenarios: &'a [BenchmarkScenario],
    name: &str,
) -> Option<&'a BenchmarkScenario> {
    scenarios.iter().find(|s| s.name == name)
}
