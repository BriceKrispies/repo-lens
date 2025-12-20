//! Typed request/response DTOs for the repo-lens API contract.
//!
//! This crate defines the stable API contract used for communication between
//! UI clients and the repo-lens backend engine.

pub mod bounds;
pub mod error;
pub mod event;
pub mod paging;
pub mod request;
pub mod response;
pub mod version;

// Re-export main types for convenience
pub use bounds::{Cursor, MaxBytes, MaxHunks, PageSize, WindowSize};
pub use error::{Error, ErrorCode};
pub use event::Event;
pub use paging::{Paging, StreamingChunk};
pub use request::Request;
pub use response::Response;
pub use version::ApiVersion;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_serialization() {
        // Test that serialization is deterministic (same input -> same output)
        let request1 = Request {
            version: ApiVersion::V0,
            id: "test-123".to_string(),
            payload: request::RequestPayload::Status(request::StatusRequest {
                repo_path: "/path/to/repo".to_string(),
            }),
        };

        let request2 = Request {
            version: ApiVersion::V0,
            id: "test-123".to_string(),
            payload: request::RequestPayload::Status(request::StatusRequest {
                repo_path: "/path/to/repo".to_string(),
            }),
        };

        let json1 = serde_json::to_string(&request1).unwrap();
        let json2 = serde_json::to_string(&request2).unwrap();

        assert_eq!(json1, json2, "Serialization should be deterministic");

        // Test round-trip
        let deserialized: Request = serde_json::from_str(&json1).unwrap();
        assert_eq!(deserialized.version, request1.version);
        assert_eq!(deserialized.id, request1.id);
    }

    #[test]
    fn test_page_size_bounds() {
        assert!(PageSize::try_from(1).is_ok());
        assert!(PageSize::try_from(1000).is_ok());
        assert!(PageSize::try_from(1001).is_err());
        assert!(PageSize::try_from(0).is_err());
    }

    #[test]
    fn test_window_size_bounds() {
        assert!(WindowSize::try_from(1).is_ok());
        assert!(WindowSize::try_from(10000).is_ok());
        assert!(WindowSize::try_from(10001).is_err());
        assert!(WindowSize::try_from(0).is_err());
    }

    #[test]
    fn test_max_bytes_bounds() {
        assert!(MaxBytes::try_from(1).is_ok());
        assert!(MaxBytes::try_from(bounds::MAX_DIFF_BYTES).is_ok());
        assert!(MaxBytes::try_from(bounds::MAX_DIFF_BYTES + 1).is_err());
        assert!(MaxBytes::try_from(0).is_err());
    }

    #[test]
    fn test_max_hunks_bounds() {
        assert!(MaxHunks::try_from(1).is_ok());
        assert!(MaxHunks::try_from(bounds::MAX_DIFF_HUNKS).is_ok());
        assert!(MaxHunks::try_from(bounds::MAX_DIFF_HUNKS + 1).is_err());
        assert!(MaxHunks::try_from(0).is_err());
    }

    #[test]
    fn test_cursor() {
        let cursor = Cursor::initial();
        assert_eq!(cursor.get(), "");

        let cursor = Cursor::from("test".to_string());
        assert_eq!(cursor.get(), "test");
    }
}
