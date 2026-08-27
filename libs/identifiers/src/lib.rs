//! # Identifiers Library
//!
//! A flexible ID generation library for Rust applications supporting ULID, UUID v7, and `NanoID` formats
//! with compile-time feature selection.
//!
//! This library provides a unified interface for generating unique identifiers with three different
//! backend implementations, each optimized for different use cases:
//!
//! - **ULID** (default): Universally Unique Lexicographically Sortable Identifier
//! - **UUID v7**: Time-ordered UUID variant with Unix timestamp precision
//! - **`NanoID`**: Compact, URL-safe unique string ID generator
//!
//! ## Feature Priority System
//!
//! When multiple features are enabled, the library follows this priority:
//! 1. ULID (highest priority)
//! 2. UUID v7
//! 3. `NanoID` (lowest priority)
//!
//! ## Quick Start
//!
//! ```rust
//! use identifiers::{generate, Id};
//!
//! // Generate a new ID using the selected backend
//! let id: Id = generate();
//! println!("Generated ID: {}", id);
//! ```
//!
//! ## Feature Selection
//!
//! Enable specific ID formats via Cargo features:
//!
//! ```toml
//! # Default (ULID)
//! identifiers = { path = "../identifiers" }
//!
//! # UUID v7 only
//! identifiers = { path = "../identifiers", default-features = false, features = ["uuid"] }
//!
//! # NanoID only
//! identifiers = { path = "../identifiers", default-features = false, features = ["nanoid"] }
//! ```
//!
//! ## Performance Characteristics
//!
//! | Format | Generation Speed | Storage | Sortability | URL-Safe |
//! |--------|------------------|---------|-------------|-----------|
//! | ULID   | High            | 26 chars| ✓          | ✓        |
//! | UUID   | Medium          | 36 chars| ✓          | ✗        |
//! | NanoID | High            | 21 chars| ✗          | ✓        |

#[cfg(feature = "ulid")]
pub mod ulid;

#[cfg(feature = "uuid")]
pub mod uuid;

#[cfg(feature = "nanoid")]
pub mod nanoid;

// Re-export the selected ID implementation based on feature priority

/// ULID has the highest priority when enabled
#[cfg(feature = "ulid")]
pub use ulid::{Id, generate};

/// UUID is selected when enabled and ULID is not
#[cfg(all(feature = "uuid", not(feature = "nanoid"), not(feature = "ulid")))]
pub use uuid::{Id, generate};

/// NanoID is selected when it's the only enabled feature
#[cfg(all(feature = "nanoid", not(feature = "uuid"), not(feature = "ulid")))]
pub use nanoid::{Id, generate};

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(feature = "ulid", feature = "uuid", feature = "nanoid"))]
    #[test]
    fn test_generate() {
        let id = generate();
        assert!(!id.is_empty());
    }

    #[cfg(feature = "ulid")]
    #[test]
    fn test_default_ulid_export() {
        let id = generate();
        // Should be a string with ULID format
        assert_eq!(id.len(), 26);
    }

    #[cfg(all(feature = "uuid", not(feature = "nanoid"), not(feature = "ulid")))]
    #[test]
    fn test_uuid_only_export() {
        let id = generate();
        // Should be a UUID
        assert_eq!(id.to_string().len(), 36);
    }

    #[cfg(all(feature = "nanoid", not(feature = "uuid"), not(feature = "ulid")))]
    #[test]
    fn test_nanoid_only_export() {
        let id = generate();
        // Should be a nanoid string
        assert_eq!(id.len(), 21);
    }
}
