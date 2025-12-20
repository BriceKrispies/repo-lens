//! Pagination and streaming support structures.

use crate::bounds::{Cursor, PageSize};
use serde::{Deserialize, Serialize};

/// Pagination parameters for list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paging {
    /// Maximum number of items to return
    pub page_size: PageSize,
    /// Cursor for resuming pagination
    pub cursor: Cursor,
}

/// A chunk in a streaming response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingChunk<T> {
    /// Sequence number for ordering chunks
    pub sequence: u64,
    /// Whether this is the final chunk
    pub is_final: bool,
    /// The chunk data
    pub data: T,
}
