//! Diff summary benchmark scenario

use criterion::{black_box, Criterion};
use rl_api::{request::*, ApiVersion, Request};
use rl_core::RepoEngine;
use std::path::Path;

#[allow(dead_code)]
pub fn bench_diff_summary(c: &mut Criterion, repo_path: &Path) {
    let engine = RepoEngine::new();
    let repo_path_str = repo_path.to_string_lossy().to_string();

    let request = Request {
        version: ApiVersion::V0,
        id: "bench-diff-summary".to_string(),
        payload: RequestPayload::DiffSummary(DiffSummaryRequest {
            repo_path: repo_path_str,
            // Use commits that exist in the Git v2.45.0 repository
            from: Some("HEAD~10".to_string()),
            to: Some("HEAD".to_string()),
            max_bytes: rl_api::MaxBytes::try_from(1024 * 1024).unwrap(),
            max_hunks: rl_api::MaxHunks::try_from(1000).unwrap(),
        }),
    };

    c.bench_function("diff_summary", |b| {
        b.iter(|| {
            let request = black_box(request.clone());
            // This calls the stubbed engine - will return "not implemented" but measures overhead
            let _result = futures::executor::block_on(engine.handle(request));
        });
    });
}
