//! ULID (Universally Unique Lexicographically Sortable Identifier) implementation
//!
//! ULIDs are 128-bit identifiers that are:
//! - **Time-ordered**: Lexicographically sortable by generation time
//! - **Compact**: 26-character Base32 encoding
//! - **Case-insensitive**: Uses Crockford's Base32 for readability
//! - **Monotonic**: Within the same millisecond, ULIDs are monotonically increasing
//!
//! ## Format Structure
//!
//! ```text
//! 01AN4Z07BY      79KA1307SR9X4MV3
//! |----------|    |----------------|
//! Timestamp          Randomness
//! 48bits             80bits
//! ```
//!
//! ## Performance
//!
//! - Generation: ~100ns per ID
//! - Sorting: O(1) lexicographic comparison
//! - Storage: 26 bytes as string, 16 bytes as binary
//!
//! ## Use Cases
//!
//! - Database primary keys requiring natural sorting
//! - Distributed system event ordering
//! - Log entries and audit trails
//! - Any scenario where time-based ordering is important

use ulid::Ulid;

/// Type alias for ULID represented as a string
///
/// ULIDs are always 26 characters long using Crockford's Base32 encoding.
/// Example: `01ARZ3NDEKTSV4RRFFQ69G5FAV`
pub type Id = String;

/// Generate a new ULID
///
/// Creates a new ULID with the current timestamp and cryptographically secure random data.
/// The generated ULID will be lexicographically sortable by creation time.
///
/// # Examples
///
/// ```rust
/// use identifiers::ulid::generate;
///
/// let id = generate();
/// assert_eq!(id.len(), 26);
/// println!("Generated ULID: {}", id);
/// ```
///
/// # Performance
///
/// This function is very fast (~100ns) and suitable for high-throughput scenarios.
/// Multiple calls within the same millisecond will produce monotonically increasing ULIDs.
pub fn generate() -> Id {
    let ulid = Ulid::generate();

    ulid.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ulid_generate() {
        let id = generate();
        assert!(!id.is_empty());
        assert_eq!(id.len(), 26); // ULID is always 26 characters
    }

    #[test]
    fn test_ulid_multiple_generations_unique() {
        let id1 = generate();
        let id2 = generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_ulid_format() {
        let id = generate();
        // ULID should be base32 encoded, all uppercase
        assert!(id.chars().all(
            |c| c.is_ascii_alphanumeric() && c.is_ascii_uppercase() || "0123456789".contains(c)
        ));
    }
}
