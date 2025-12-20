//! Status benchmark scenario

use criterion::{black_box, Criterion};
use rl_api::{request::*, ApiVersion, Request};
use rl_core::RepoEngine;
use std::path::Path;

#[allow(dead_code)]
pub fn bench_status(c: &mut Criterion, repo_path: &Path) {
    let engine = RepoEngine::new();
    let repo_path_str = repo_path.to_string_lossy().to_string();

    let request = Request {
        version: ApiVersion::V0,
        id: "bench-status".to_string(),
        payload: RequestPayload::Status(StatusRequest {
            repo_path: repo_path_str,
        }),
    };

    c.bench_function("status", |b| {
        b.iter(|| {
            let request = black_box(request.clone());
            // This calls the stubbed engine - will return "not implemented" but measures overhead
            let _result = futures::executor::block_on(engine.handle(request));
        });
    });
}
