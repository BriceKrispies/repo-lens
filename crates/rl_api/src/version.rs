//! API versioning for the repo-lens contract.

use serde::{Deserialize, Serialize};

/// Current API version identifier.
/// Breaking changes require bumping this version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiVersion {
    /// Version 0 - initial stable contract
    #[serde(rename = "v0")]
    V0,
}

impl Default for ApiVersion {
    fn default() -> Self {
        Self::V0
    }
}
