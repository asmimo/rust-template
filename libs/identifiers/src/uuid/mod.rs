//! UUID v7 (Time-ordered Universal Unique Identifier) implementation
//!
//! UUID v7 is the newest UUID variant (RFC 9562) that combines:
//! - **Time-ordering**: Unix timestamp in the first 48 bits
//! - **Randomness**: Cryptographically secure random data
//! - **Compatibility**: Standard UUID format (8-4-4-4-12)
//! - **Sortability**: Natural chronological ordering
//!
//! ## Format Structure
//!
//! ```text
//! 017F22E2-79B0-7CC3-98C4-DC0C0C07398F
//! |------------|  |  |----------------|
//! Unix Timestamp  Ver    Random Data
//!    48 bits       4        74 bits
//! ```
//!
//! ## Advantages over UUID v4
//!
//! - **Database performance**: Better for B-tree indexes (ordered inserts)
//! - **Debugging**: Human-readable creation time
//! - **Distributed systems**: Natural ordering across nodes
//! - **Reduced fragmentation**: Sequential nature reduces index fragmentation
//!
//! ## Use Cases
//!
//! - Database primary keys where UUID compatibility is required
//! - Systems requiring both uniqueness and time-ordering
//! - Migration from UUID v4 without changing column types
//! - Inter-service communication requiring standard UUIDs

pub use uuid::*;

/// Type alias for UUID v7
///
/// UUIDs are always 36 characters when formatted as strings (with hyphens)
/// or 32 characters without hyphens. Example: `017f22e2-79b0-7cc3-98c4-dc0c0c07398f`
pub type Id = uuid::Uuid;

/// Generate a new UUID v7
///
/// Creates a new UUID v7 with the current Unix timestamp and cryptographically secure
/// random data. The generated UUID will be naturally sortable by creation time.
///
/// # Examples
///
/// ```rust
/// use identifiers::uuid::generate;
///
/// let id = generate();
/// assert_eq!(id.get_version(), Some(uuid::Version::SortRand)); // v7
/// println!("Generated UUID: {}", id);
/// ```
///
/// # Performance
///
/// UUID v7 generation is slightly slower than ULID (~200ns) due to additional
/// formatting and validation, but still suitable for most use cases.
///
/// # Compatibility
///
/// Fully compatible with existing UUID infrastructure, parsers, and databases.
pub fn generate() -> Id {
    uuid::Uuid::now_v7()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_generate() {
        let id = generate();
        assert!(!id.to_string().is_empty());
        assert_eq!(id.get_version(), Some(uuid::Version::SortRand)); // v7
    }

    #[test]
    fn test_uuid_multiple_generations_unique() {
        let id1 = generate();
        let id2 = generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_uuid_format() {
        let id = generate();
        let id_str = id.to_string();

        // UUID v7 format: 8-4-4-4-12 characters
        assert_eq!(id_str.len(), 36);
        assert_eq!(id_str.chars().filter(|&c| c == '-').count(), 4);

        // Check format with regex-like validation
        let parts: Vec<&str> = id_str.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }
}
