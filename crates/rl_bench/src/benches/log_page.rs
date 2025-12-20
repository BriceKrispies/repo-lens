//! Log page benchmark scenario

use criterion::{black_box, Criterion};
use rl_api::{request::*, ApiVersion, Request};
use rl_core::RepoEngine;
use std::path::Path;

#[allow(dead_code)]
pub fn bench_log_page(c: &mut Criterion, repo_path: &Path) {
    let engine = RepoEngine::new();
    let repo_path_str = repo_path.to_string_lossy().to_string();

    let request = Request {
        version: ApiVersion::V0,
        id: "bench-log-page".to_string(),
        payload: RequestPayload::Log(LogRequest {
            repo_path: repo_path_str,
            paging: rl_api::Paging {
                page_size: rl_api::PageSize::try_from(200).unwrap(),
                cursor: rl_api::Cursor::initial(),
            },
            revision_range: None,
        }),
    };

    c.bench_function("log_page", |b| {
        b.iter(|| {
            let request = black_box(request.clone());
            // This calls the stubbed engine - will return "not implemented" but measures overhead
            let _result = futures::executor::block_on(engine.handle(request));
        });
    });
}
