use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{info_span, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn init_telemetry(filter: Option<&str>, json: bool) {
    // Default to "off" if no filter specified, so JSON output is clean by default
    let filter = filter.unwrap_or("off");
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter));

    let registry = tracing_subscriber::registry().with(filter);

    let fmt_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr);

    // Use try_init to avoid panicking if already initialized
    if json {
        let _ = registry.with(fmt_layer.json()).try_init();
    } else {
        let _ = registry.with(fmt_layer).try_init();
    }
}

pub fn new_request_id() -> String {
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("req_{}_{}", timestamp, counter)
}

#[derive(Debug)]
pub struct RequestSpan {
    span: Span,
}

impl RequestSpan {
    pub fn new(request_id: &str, repo_path: &str, request_type: &str) -> Self {
        let span = info_span!(
            "request",
            request_id = request_id,
            repo_path = repo_path,
            request_type = request_type
        );
        Self { span }
    }

    pub fn enter(&self) -> Span {
        self.span.clone()
    }
}

#[macro_export]
macro_rules! step {
    ($name:expr, $block:block) => {{
        let span = tracing::info_span!($name);
        let _enter = span.enter();

        async {
            let start = std::time::Instant::now();
            let result = $block;
            let elapsed_ms = start.elapsed().as_nanos() as f64 / 1_000_000.0;

            match &result {
                Ok(_) => {
                    tracing::info!(elapsed_ms = elapsed_ms, "step completed");
                }
                Err(e) => {
                    tracing::error!(elapsed_ms = elapsed_ms, error = %e, "step failed");
                }
            }

            result
        }
        .await
    }};
}
