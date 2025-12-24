//! Bench harness library for repo-lens.
//!
//! This library provides the core benchmarking infrastructure used by the
//! repo-lens-bench binary.

pub mod benches;
pub mod datasets;
pub mod oracle;
pub mod regression;
pub mod scenarios;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_oracle_git_cli_rev_parse() {
        let dataset_path = Path::new("target/rl_bench/datasets/git");

        if !dataset_path.exists() {
            return; // Skip test if dataset doesn't exist
        }

        let git_cli = oracle::git_cli::GitCli::new(dataset_path);
        let result = git_cli.run(&["rev-parse", "HEAD"]);

        match result {
            Ok(output) => {
                let lines = oracle::normalize::normalize_lines(&output.stdout);
                assert_eq!(
                    lines.len(),
                    1,
                    "Expected exactly 1 line from rev-parse HEAD"
                );
                assert!(!lines[0].is_empty(), "Expected non-empty SHA");
                assert!(lines[0].len() >= 4, "Expected reasonable SHA length");
            }
            Err(e) => {
                panic!("Git command failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_oracle_status_correctness() {
        use rl_fixtures::synth_repo::SynthRepo;

        let synth = match SynthRepo::ensure("oracle_status") {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!("Failed to create synthetic repo: {}", e);
                return;
            }
        };

        if let Err(e) = synth.modify_working_tree("a.txt", "modified line\n") {
            eprintln!("Failed to modify working tree: {}", e);
            return;
        }

        let git_cli = oracle::git_cli::GitCli::new(&synth.path);
        let oracle_result = git_cli.run(&["status", "--porcelain=v1"]);

        let oracle_output = match oracle_result {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Skipping oracle status test: git command failed: {}", e);
                return;
            }
        };

        let engine = rl_core::RepoEngine::new();
        let request = rl_api::Request {
            version: rl_api::ApiVersion::V0,
            id: "oracle-test".to_string(),
            payload: rl_api::request::RequestPayload::Status(rl_api::request::StatusRequest {
                repo_path: synth.path.to_string_lossy().to_string(),
            }),
        };

        let response = engine.handle(request).await;

        let status_view = match response.result {
            Ok(rl_api::response::ResponsePayload::Status(status)) => status,
            Ok(other) => panic!("Expected Status response, got {:?}", other),
            Err(e) => panic!("Engine returned error: {}", e),
        };

        let oracle_lines = oracle::normalize::normalize_lines(&oracle_output.stdout);

        let mut engine_lines_raw = Vec::new();

        for file in &status_view.index.staged {
            engine_lines_raw.push(format!("A  {}", file));
        }

        for file in &status_view.workdir.modified {
            engine_lines_raw.push(format!(" M {}", file));
        }

        for file in &status_view.workdir.deleted {
            engine_lines_raw.push(format!(" D {}", file));
        }

        for file in &status_view.workdir.untracked {
            engine_lines_raw.push(format!("?? {}", file));
        }

        let engine_output_str = engine_lines_raw.join("\n");
        let engine_lines = oracle::normalize::normalize_lines(&engine_output_str);

        let oracle_normalized = oracle::normalize::sort_stable(oracle_lines);
        let engine_normalized = oracle::normalize::sort_stable(engine_lines);

        match oracle::compare::compare_lines(&oracle_normalized, &engine_normalized) {
            Ok(_) => {
                eprintln!("✓ Oracle status test passed");
            }
            Err(diff) => {
                eprintln!("Oracle status test FAILED");
                eprintln!(
                    "Expected {} lines, got {}",
                    diff.expected_len, diff.actual_len
                );
                if let Some(idx) = diff.first_mismatch {
                    eprintln!("First mismatch at line {}", idx);
                }
                eprintln!("\nExpected (first 10):");
                for line in &diff.expected_sample {
                    eprintln!("  {}", line);
                }
                eprintln!("\nActual (first 10):");
                for line in &diff.actual_sample {
                    eprintln!("  {}", line);
                }
                panic!("Oracle comparison failed");
            }
        }
    }

    #[tokio::test]
    async fn test_oracle_diff_summary_c0_c1() {
        use rl_fixtures::synth_repo::SynthRepo;

        let synth = match SynthRepo::ensure("oracle_diff") {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!("Failed to create synthetic repo: {}", e);
                return;
            }
        };

        let git_cli = oracle::git_cli::GitCli::new(&synth.path);

        let name_status = git_cli
            .run(&["diff", "--name-status", "-M", "C0..C1"])
            .unwrap();

        let engine = rl_core::RepoEngine::new();
        let request = rl_api::Request {
            version: rl_api::ApiVersion::V0,
            id: "oracle-diff-test".to_string(),
            payload: rl_api::request::RequestPayload::DiffSummary(
                rl_api::request::DiffSummaryRequest {
                    repo_path: synth.path.to_string_lossy().to_string(),
                    from: Some("C0".to_string()),
                    to: Some("C1".to_string()),
                    max_bytes: rl_api::MaxBytes::try_from(1024 * 1024).unwrap(),
                    max_hunks: rl_api::MaxHunks::try_from(1000).unwrap(),
                },
            ),
        };

        let response = engine.handle(request).await;

        let diff_summary = match response.result {
            Ok(rl_api::response::ResponsePayload::DiffSummary(diff)) => diff,
            Ok(other) => panic!("Expected DiffSummary response, got {:?}", other),
            Err(e) => panic!("Engine returned error: {}", e),
        };

        let mut oracle_name_status: Vec<String> = name_status
            .stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect();
        oracle_name_status.sort();

        let mut engine_name_status: Vec<String> = diff_summary
            .changes
            .iter()
            .map(|c| {
                let status_char = match c.change_type {
                    rl_api::response::ChangeType::Added => 'A',
                    rl_api::response::ChangeType::Modified => 'M',
                    rl_api::response::ChangeType::Deleted => 'D',
                    rl_api::response::ChangeType::Renamed => 'R',
                };
                if let Some(old_path) = &c.old_path {
                    format!("{}\t{}\t{}", status_char, old_path, c.path)
                } else {
                    format!("{}\t{}", status_char, c.path)
                }
            })
            .collect();
        engine_name_status.sort();

        match oracle::compare::compare_lines(&oracle_name_status, &engine_name_status) {
            Ok(_) => {
                eprintln!("✓ Oracle diff C0..C1 test passed");
            }
            Err(diff) => {
                eprintln!("Oracle diff C0..C1 test FAILED");
                eprintln!(
                    "Expected {} lines, got {}",
                    diff.expected_len, diff.actual_len
                );
                panic!("Oracle comparison failed");
            }
        }

        assert_eq!(diff_summary.files_changed, 2);
    }

    #[tokio::test]
    async fn test_oracle_diff_summary_c1_c2() {
        use rl_fixtures::synth_repo::SynthRepo;

        let synth = match SynthRepo::ensure("oracle_diff") {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!("Failed to create synthetic repo: {}", e);
                return;
            }
        };

        let engine = rl_core::RepoEngine::new();
        let request = rl_api::Request {
            version: rl_api::ApiVersion::V0,
            id: "oracle-diff-test".to_string(),
            payload: rl_api::request::RequestPayload::DiffSummary(
                rl_api::request::DiffSummaryRequest {
                    repo_path: synth.path.to_string_lossy().to_string(),
                    from: Some("C1".to_string()),
                    to: Some("C2".to_string()),
                    max_bytes: rl_api::MaxBytes::try_from(1024 * 1024).unwrap(),
                    max_hunks: rl_api::MaxHunks::try_from(1000).unwrap(),
                },
            ),
        };

        let response = engine.handle(request).await;

        let diff_summary = match response.result {
            Ok(rl_api::response::ResponsePayload::DiffSummary(diff)) => diff,
            Ok(other) => panic!("Expected DiffSummary response, got {:?}", other),
            Err(e) => panic!("Engine returned error: {}", e),
        };

        let has_rename = diff_summary
            .changes
            .iter()
            .any(|c| matches!(c.change_type, rl_api::response::ChangeType::Renamed));

        assert!(has_rename, "Expected rename in C1..C2");

        eprintln!("✓ Oracle diff C1..C2 test passed");
    }

    #[tokio::test]
    async fn test_oracle_diff_summary_c2_c3() {
        use rl_fixtures::synth_repo::SynthRepo;

        let synth = match SynthRepo::ensure("oracle_diff") {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!("Failed to create synthetic repo: {}", e);
                return;
            }
        };

        let engine = rl_core::RepoEngine::new();
        let request = rl_api::Request {
            version: rl_api::ApiVersion::V0,
            id: "oracle-diff-test".to_string(),
            payload: rl_api::request::RequestPayload::DiffSummary(
                rl_api::request::DiffSummaryRequest {
                    repo_path: synth.path.to_string_lossy().to_string(),
                    from: Some("C2".to_string()),
                    to: Some("C3".to_string()),
                    max_bytes: rl_api::MaxBytes::try_from(1024 * 1024).unwrap(),
                    max_hunks: rl_api::MaxHunks::try_from(1000).unwrap(),
                },
            ),
        };

        let response = engine.handle(request).await;

        let diff_summary = match response.result {
            Ok(rl_api::response::ResponsePayload::DiffSummary(diff)) => diff,
            Ok(other) => panic!("Expected DiffSummary response, got {:?}", other),
            Err(e) => panic!("Engine returned error: {}", e),
        };

        let has_delete = diff_summary
            .changes
            .iter()
            .any(|c| matches!(c.change_type, rl_api::response::ChangeType::Deleted));
        let has_add = diff_summary
            .changes
            .iter()
            .any(|c| matches!(c.change_type, rl_api::response::ChangeType::Added));

        assert!(has_delete, "Expected deletion in C2..C3");
        assert!(has_add, "Expected addition in C2..C3");

        eprintln!("✓ Oracle diff C2..C3 test passed");
    }
}
