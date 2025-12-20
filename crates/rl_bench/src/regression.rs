//! Regression detection for benchmark results.
//!
//! This module provides simple regression analysis by comparing benchmark runs
//! against saved baselines and detecting performance regressions.

use crate::scenarios::BenchmarkResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Regression threshold (20% increase in wall time)
const REGRESSION_THRESHOLD: f64 = 0.20;

/// Regression analysis result
#[derive(Debug, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    /// Whether any regressions were detected
    pub has_regressions: bool,
    /// Analysis for each scenario
    pub scenario_results: Vec<ScenarioRegression>,
}

/// Regression analysis for a single scenario
#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioRegression {
    /// Scenario name
    pub scenario: String,
    /// Baseline result
    pub baseline: BenchmarkResult,
    /// Current result
    pub current: BenchmarkResult,
    /// Relative change (positive = regression, negative = improvement)
    pub relative_change: f64,
    /// Whether this is a regression
    pub is_regression: bool,
    /// Human-readable status
    pub status: String,
}

impl RegressionAnalysis {
    /// Analyze regressions by comparing current results against a baseline
    pub fn analyze(
        baseline_results: &[BenchmarkResult],
        current_results: &[BenchmarkResult],
    ) -> Self {
        let mut scenario_results = Vec::new();
        let mut has_regressions = false;

        // Create lookup map for baseline results
        let baseline_map: HashMap<String, &BenchmarkResult> = baseline_results
            .iter()
            .map(|r| (r.scenario.clone(), r))
            .collect();

        for current in current_results {
            if let Some(baseline) = baseline_map.get(&current.scenario) {
                let relative_change = if baseline.wall_time_ns > 0 {
                    (current.wall_time_ns as f64 - baseline.wall_time_ns as f64)
                        / baseline.wall_time_ns as f64
                } else {
                    0.0
                };

                let is_regression = relative_change > REGRESSION_THRESHOLD;

                if is_regression {
                    has_regressions = true;
                }

                let status = if is_regression {
                    format!("REGRESSION: {:.1}% increase", relative_change * 100.0)
                } else if relative_change < -REGRESSION_THRESHOLD {
                    format!("IMPROVEMENT: {:.1}% decrease", -relative_change * 100.0)
                } else {
                    format!("STABLE: {:.1}% change", relative_change * 100.0)
                };

                scenario_results.push(ScenarioRegression {
                    scenario: current.scenario.clone(),
                    baseline: (*baseline).clone(),
                    current: current.clone(),
                    relative_change,
                    is_regression,
                    status,
                });
            }
        }

        Self {
            has_regressions,
            scenario_results,
        }
    }

    /// Exit with error code if regressions detected
    pub fn exit_on_regression(&self) -> ! {
        if self.has_regressions {
            eprintln!("❌ Performance regressions detected!");
            for result in &self.scenario_results {
                if result.is_regression {
                    eprintln!("  {}: {}", result.scenario, result.status);
                }
            }
            std::process::exit(1);
        } else {
            println!("✅ No performance regressions detected.");
            std::process::exit(0);
        }
    }
}

/// Load benchmark results from a JSON file
pub fn load_baseline(path: &Path) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let results: Vec<BenchmarkResult> = serde_json::from_str(&content)?;
    Ok(results)
}

/// Save benchmark results to a JSON file
#[allow(dead_code)]
pub fn save_baseline(
    path: &Path,
    results: &[BenchmarkResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(results)?;
    fs::write(path, json)?;
    Ok(())
}

/// Generate a default baseline filename with timestamp
#[allow(dead_code)]
pub fn default_baseline_name() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("baseline-{}.json", timestamp)
}
