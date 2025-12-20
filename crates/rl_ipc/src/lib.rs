//! Transport layer for repo-lens engine with JSON-RPC over stdio.
//!
//! This crate provides IPC transport that maps rl_api messages to rl_core calls.

use rl_api::{Request, Response};
use rl_core::RepoEngine;
use std::io::{self, Write};
use tokio::sync::mpsc;

/// IPC server that handles JSON-RPC over stdio.
pub struct IpcServer {
    /// The repo engine
    engine: RepoEngine,
}

impl IpcServer {
    /// Create a new IPC server with the given engine.
    pub fn new(engine: RepoEngine) -> Self {
        Self { engine }
    }

    /// Run the IPC server, reading from stdin and writing to stdout.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut lines = stdin.lines();

        loop {
            // Read a line from stdin
            let line = match lines.next() {
                Some(Ok(line)) => line,
                Some(Err(e)) => {
                    eprintln!("Error reading from stdin: {}", e);
                    continue;
                }
                None => break, // EOF
            };

            // Parse the request
            let request: Request = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    // Send error response
                    let error_response = Response {
                        id: "unknown".to_string(),
                        result: Err(rl_api::Error::new(
                            rl_api::ErrorCode::InvalidRequest,
                            format!("Failed to parse request: {}", e),
                        )),
                    };
                    let response_json = serde_json::to_string(&error_response)?;
                    writeln!(stdout, "{}", response_json)?;
                    stdout.flush()?;
                    continue;
                }
            };

            // Handle the request
            let response = self.engine.handle(request).await;

            // Send the response
            let response_json = serde_json::to_string(&response)?;
            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        Ok(())
    }
}

/// IPC client for communicating with the server.
pub struct IpcClient {
    /// Channel sender for requests
    #[allow(dead_code)]
    request_tx: mpsc::UnboundedSender<Request>,
    /// Channel receiver for responses
    #[allow(dead_code)]
    response_rx: mpsc::UnboundedReceiver<Response>,
}

#[allow(clippy::new_without_default)]
impl IpcClient {
    /// Create a new IPC client (stub implementation).
    pub fn new() -> Self {
        let (request_tx, _request_rx) = mpsc::unbounded_channel();
        let (_response_tx, response_rx) = mpsc::unbounded_channel();

        Self {
            request_tx,
            response_rx,
        }
    }

    /// Send a request and get a response (stub implementation).
    pub async fn send_request(&mut self, _request: Request) -> Result<Response, rl_api::Error> {
        Err(rl_api::Error::new(
            rl_api::ErrorCode::Internal,
            "IPC client not implemented",
        ))
    }
}

/// Transport configuration.
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Buffer size for reading
    pub buffer_size: usize,
    /// Timeout for operations
    pub timeout_ms: u64,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            buffer_size: 8192,
            timeout_ms: 30000, // 30 seconds
        }
    }
}
