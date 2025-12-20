//! Bench harness for repo-lens with performance testing.
//!
//! This binary runs performance benchmarks against the repo-lens engine
//! to ensure queries meet performance budgets.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Instant;

mod benches;
mod datasets;
mod regression;
mod scenarios;

use datasets::{DatasetManifest, DatasetResolver};
use regression::{default_baseline_name, load_baseline, save_baseline, RegressionAnalysis};
use scenarios::{
    generate_scenarios, BenchmarkResult, BenchmarkRun, DatasetInfo, SentinelResult, TimingInfo,
};

#[derive(Parser)]
#[command(name = "repo-lens-bench")]
#[command(about = "Performance benchmarking harness for repo-lens")]
#[command(version)]
struct Cli {
    /// Log filter (e.g., repo_lens=debug)
    #[arg(long)]
    log: Option<String>,

    /// Output logs as JSON
    #[arg(long)]
    log_json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run benchmarks against datasets
    Run {
        /// Dataset to use (default: git)
        #[arg(long, default_value = "git")]
        dataset: String,

        /// Output file for results (JSON)
        #[arg(long)]
        output: Option<PathBuf>,

        /// Scenarios to run (default: all)
        #[arg(long)]
        scenarios: Option<Vec<String>>,

        /// Budget in milliseconds for warm average timing
        #[arg(long)]
        budget_ms: Option<f64>,
    },

    /// Baseline operations
    Baseline {
        #[command(subcommand)]
        command: BaselineCommands,
    },

    /// Compare current results against a baseline (legacy)
    Compare {
        /// Path to baseline JSON file
        baseline: PathBuf,

        /// Path to current results JSON file
        current: PathBuf,
    },

    /// List available datasets
    ListDatasets,
}

#[derive(Subcommand)]
enum BaselineCommands {
    /// Save current run as baseline
    Save {
        /// Output baseline file path
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Compare current run against baseline
    Compare {
        /// Path to baseline JSON file
        baseline: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    rl_core::telemetry::init_telemetry(cli.log.as_deref(), cli.log_json);

    match cli.command {
        Commands::Run {
            dataset,
            output,
            scenarios,
            budget_ms,
        } => {
            run_benchmarks(&dataset, output, scenarios, budget_ms).await?;
        }
        Commands::Baseline { command } => match command {
            BaselineCommands::Save { output } => {
                run_and_save_baseline(output).await?;
            }
            BaselineCommands::Compare { baseline } => {
                compare_against_baseline(&baseline).await?;
            }
        },
        Commands::ListDatasets => {
            list_datasets()?;
        }
        Commands::Compare { baseline, current } => {
            compare_baselines(&baseline, &current)?;
        }
    }

    Ok(())
}

async fn run_benchmarks(
    dataset_name: &str,
    output_path: Option<PathBuf>,
    scenario_filter: Option<Vec<String>>,
    budget_ms: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load dataset manifest and find requested dataset
    let manifest = DatasetManifest::load()?;
    let dataset = manifest
        .find_by_name(dataset_name)
        .ok_or_else(|| format!("Dataset '{}' not found", dataset_name))?;

    // Resolve dataset (just check path, don't clone for sentinel)
    let resolver = DatasetResolver::new()?;
    let dataset_path = resolver.cache_dir().join(&dataset.name);
    let dataset_exists = dataset_path.exists();

    // Generate scenarios for this dataset
    let all_scenarios = generate_scenarios(&dataset_path);
    let scenarios_to_run: Vec<_> = if let Some(filter) = scenario_filter {
        all_scenarios
            .into_iter()
            .filter(|s| filter.contains(&s.name))
            .collect()
    } else {
        // Default to engine_overhead for sentinel benchmark
        all_scenarios
            .into_iter()
            .filter(|s| s.name == "engine_overhead")
            .collect()
    };

    if scenarios_to_run.is_empty() {
        return Err("No scenarios to run".into());
    }

    eprintln!("Running sentinel benchmark...");

    // Run sentinel benchmark
    let engine = rl_core::RepoEngine::new();
    let mut results = Vec::new();

    for scenario in &scenarios_to_run {
        eprintln!("Running scenario: {}", scenario.name);

        let result = run_sentinel_scenario(
            &engine,
            scenario,
            dataset,
            &dataset_path,
            dataset_exists,
            budget_ms,
        )
        .await?;
        results.push(result);
    }

    // Check for failures before consuming results
    let has_failure = results.iter().any(|r| r.status == "fail");

    // For single scenario (sentinel), output the result directly
    if results.len() == 1 {
        let json_output = serde_json::to_string_pretty(&results[0])?;
        match output_path {
            Some(path) => {
                std::fs::write(&path, &json_output)?;
                eprintln!("Results saved to {}", path.display());
            }
            None => {
                println!("{}", json_output);
            }
        }
    } else {
        // Fallback to old format for multiple scenarios
        let run = BenchmarkRun {
            timestamp: chrono::Utc::now().to_rfc3339(),
            dataset: dataset.name.clone(),
            results: results
                .into_iter()
                .map(|sr| BenchmarkResult {
                    scenario: sr.scenario,
                    wall_time_ns: (sr.timings.cold_ms * 1_000_000.0) as u64, // Convert to ns
                    success: sr.status == "pass",
                    error: None,
                })
                .collect(),
        };

        let json_output = serde_json::to_string_pretty(&run)?;
        match output_path {
            Some(path) => {
                std::fs::write(&path, &json_output)?;
                eprintln!("Results saved to {}", path.display());
            }
            None => {
                println!("{}", json_output);
            }
        }
    }

    // Exit with error code if any scenario failed
    if has_failure {
        std::process::exit(1);
    }

    Ok(())
}

async fn run_sentinel_scenario(
    engine: &rl_core::RepoEngine,
    scenario: &scenarios::BenchmarkScenario,
    dataset: &datasets::Dataset,
    dataset_path: &std::path::Path,
    dataset_exists: bool,
    budget_ms: Option<f64>,
) -> Result<SentinelResult, Box<dyn std::error::Error>> {
    const WARM_ITERATIONS: usize = 200;

    // Cold run (first execution)
    let start = Instant::now();
    let response = engine.handle(scenario.request.clone()).await;
    let cold_time_ms = start.elapsed().as_nanos() as f64 / 1_000_000.0;

    // Ensure response is used to prevent optimization
    let _serialized = serde_json::to_string(&response)?;

    // Warm runs - time the entire loop as one block
    let warm_start = Instant::now();
    for _ in 0..WARM_ITERATIONS {
        let response = engine.handle(scenario.request.clone()).await;
        // Ensure response is used to prevent optimization
        let _serialized = serde_json::to_string(&response)?;
    }
    let warm_total_ms = warm_start.elapsed().as_nanos() as f64 / 1_000_000.0;
    let warm_avg_ms = warm_total_ms / WARM_ITERATIONS as f64;

    // Determine status and reason
    let (status, reason) = if let Some(budget) = budget_ms {
        if warm_avg_ms > budget {
            ("fail".to_string(), Some("budget_exceeded".to_string()))
        } else {
            ("pass".to_string(), None)
        }
    } else {
        // Sentinel benchmarks always pass unless there's a hard error
        ("pass".to_string(), None)
    };

    let result = SentinelResult {
        dataset: DatasetInfo {
            name: dataset.name.clone(),
            url: dataset.url.clone(),
            rev: dataset.revision.clone(),
            path: dataset_path.to_string_lossy().to_string(),
            exists: dataset_exists,
        },
        scenario: scenario.name.clone(),
        timings: TimingInfo {
            cold_ms: cold_time_ms,
            warm_total_ms,
            warm_avg_ms,
            iterations: WARM_ITERATIONS,
        },
        status,
        reason,
    };

    Ok(result)
}

fn compare_baselines(
    baseline_path: &std::path::Path,
    current_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let baseline_results = load_baseline(baseline_path)?;
    let current_results = load_baseline(current_path)?;

    let analysis = RegressionAnalysis::analyze(&baseline_results, &current_results);

    // Output analysis as JSON
    let json_output = serde_json::to_string_pretty(&analysis)?;
    println!("{}", json_output);

    // Exit with error if regressions detected
    if analysis.has_regressions {
        analysis.exit_on_regression();
    }

    Ok(())
}

fn list_datasets() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = datasets::DatasetManifest::load()?;
    let resolver = DatasetResolver::new()?;
    let cached = resolver.list_cached()?;

    let output = serde_json::json!({
        "available": manifest.datasets,
        "cached": cached,
        "cache_dir": resolver.cache_dir().to_string_lossy()
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn run_and_save_baseline(
    output_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path =
        output_path.unwrap_or_else(|| PathBuf::from("crates/rl_bench/baselines/local.json"));

    // Run benchmark and save as baseline
    run_benchmarks("git", Some(output_path.clone()), None, None).await?;

    eprintln!("Baseline saved to {}", output_path.display());
    Ok(())
}

async fn compare_against_baseline(
    baseline_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load baseline
    let baseline_content = std::fs::read_to_string(baseline_path)?;
    let baseline: SentinelResult = serde_json::from_str(&baseline_content)?;

    // Run current benchmark
    let manifest = DatasetManifest::load()?;
    let dataset = manifest
        .find_by_name("git")
        .ok_or("Dataset 'git' not found")?;

    let resolver = DatasetResolver::new()?;
    let dataset_path = resolver.cache_dir().join(&dataset.name);
    let dataset_exists = dataset_path.exists();

    let scenarios = generate_scenarios(&dataset_path);
    let scenario = scenarios
        .into_iter()
        .find(|s| s.name == "engine_overhead")
        .ok_or("engine_overhead scenario not found")?;

    let engine = rl_core::RepoEngine::new();
    let current = run_sentinel_scenario(
        &engine,
        &scenario,
        dataset,
        &dataset_path,
        dataset_exists,
        None,
    )
    .await?;

    // Compare results using warm_avg_ms
    let regression_threshold = 0.20; // 20%
    let avg_regression =
        (current.timings.warm_avg_ms - baseline.timings.warm_avg_ms) / baseline.timings.warm_avg_ms;

    let has_regression = avg_regression > regression_threshold;

    // Create result with status and reason
    let status = if has_regression { "fail" } else { "pass" };
    let reason = if has_regression {
        Some("regression".to_string())
    } else {
        None
    };

    let comparison_result = serde_json::json!({
        "status": status,
        "reason": reason,
        "baseline": baseline,
        "current": current,
        "comparison": {
            "avg_regression": avg_regression,
            "has_regression": has_regression,
            "threshold": regression_threshold
        }
    });

    println!("{}", serde_json::to_string_pretty(&comparison_result)?);

    if has_regression {
        std::process::exit(1);
    }

    Ok(())
}

#[allow(dead_code)]
fn save_as_baseline(
    results_path: &std::path::Path,
    output_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = load_baseline(results_path)?;

    let output_path = output_path.unwrap_or_else(|| PathBuf::from(default_baseline_name()));
    save_baseline(&output_path, &results)?;

    eprintln!("Baseline saved to {}", output_path.display());
    Ok(())
}
