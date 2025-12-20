//! Typed error model for the repo-lens API.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Typed error codes with categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    // Request validation errors
    InvalidRequest,

    // Repository errors
    RepoNotFound,

    // Backend errors
    GitBackendError,

    // Operation conflicts
    Conflict,

    // Authentication/authorization
    AuthRequired,

    // Cancellation
    OperationCanceled,

    // Timeouts
    Timeout,

    // Internal errors
    Internal,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest => write!(f, "invalid_request"),
            Self::RepoNotFound => write!(f, "repo_not_found"),
            Self::GitBackendError => write!(f, "git_backend_error"),
            Self::Conflict => write!(f, "conflict"),
            Self::AuthRequired => write!(f, "auth_required"),
            Self::OperationCanceled => write!(f, "operation_canceled"),
            Self::Timeout => write!(f, "timeout"),
            Self::Internal => write!(f, "internal"),
        }
    }
}

/// Structured error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    /// Error code
    pub code: ErrorCode,
    /// Human-readable message
    pub message: String,
    /// Optional remediation hints
    pub remediation: Option<String>,
    /// Optional additional context
    pub details: Option<serde_json::Value>,
}

impl Error {
    /// Create a new error with the given code and message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            remediation: None,
            details: None,
        }
    }

    /// Add remediation hints.
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    /// Add additional details.
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}
