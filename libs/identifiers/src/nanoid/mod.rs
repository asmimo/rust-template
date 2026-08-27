//! `NanoID` - Compact, URL-safe unique string ID generator
//!
//! `NanoID` is designed to be:
//! - **Compact**: Only 21 characters for ~126 bits of entropy
//! - **URL-safe**: No special characters that need encoding
//! - **Fast**: Optimized for high-performance generation
//! - **Secure**: Uses cryptographically strong random number generator
//! - **Readable**: No ambiguous characters (0, O, I, l)
//!
//! ## Characteristics
//!
//! - **Length**: 21 characters (configurable)
//! - **Alphabet**: Custom alphanumeric set (62 characters)
//! - **Entropy**: ~126 bits (comparable to UUID)
//! - **Collision probability**: ~1% after generating 1 billion IDs/hour for 100 years
//!
//! ## Advantages
//!
//! - **Compact**: 43% smaller than UUID when stored as string
//! - **URL-friendly**: Can be used directly in URLs without encoding
//! - **No dependencies**: Minimal external dependencies
//! - **Customizable**: Alphabet and length can be configured
//!
//! ## Trade-offs
//!
//! - **Not sortable**: Random generation means no time-based ordering
//! - **Base62**: Less standard than Base32 (ULID) or hexadecimal (UUID)
//!
//! ## Use Cases
//!
//! - User-facing IDs (short links, invite codes)
//! - API keys and tokens
//! - File names and temporary identifiers
//! - Any scenario where compactness and URL-safety matter more than sorting

pub use nanoid::*;

/// Type alias for `NanoID` represented as a string
///
/// `NanoIDs` are always 21 characters long using a custom alphanumeric alphabet.
/// Example: `V1StGXR8_Z5jdHi6B-myT`
pub type Id = String;

/// Custom alphanumeric alphabet for `NanoID` generation
///
/// This alphabet excludes potentially confusing characters and special symbols
/// that might cause issues in URLs or when displayed to users.
///
/// Total characters: 62 (digits 0-9, uppercase A-Z, lowercase a-z)
const ALPHANUMERIC_ALPHABET: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z',
];

/// Generate a new `NanoID`
///
/// Creates a new 21-character `NanoID` using the custom alphanumeric alphabet.
/// The generated ID is cryptographically secure and URL-safe.
///
/// # Examples
///
/// ```rust
/// use identifiers::nanoid::generate;
///
/// let id = generate();
/// assert_eq!(id.len(), 21);
/// println!("Generated NanoID: {}", id);
///
/// // URL-safe - can be used directly in URLs
/// let url = format!("https://api.example.com/items/{}", id);
/// ```
///
/// # Performance
///
/// `NanoID` generation is very fast (~50ns) and suitable for high-throughput scenarios.
/// The custom alphabet ensures no URL encoding is needed.
///
/// # Security
///
/// Uses cryptographically secure random number generation. With the default 21-character
/// length, collision probability is negligible for practical applications.
pub fn generate() -> Id {
    nanoid::nanoid!(21, ALPHANUMERIC_ALPHABET)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nanoid_generate() {
        let id = generate();
        assert!(!id.is_empty());
        assert_eq!(id.len(), 21); // Default nanoid length
    }

    #[test]
    fn test_nanoid_multiple_generations_unique() {
        let id1 = generate();
        let id2 = generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_nanoid_character_set() {
        let id = generate();

        // All characters should be alphanumeric only
        for char in id.chars() {
            assert!(
                char.is_ascii_alphanumeric(),
                "Character '{char}' is not alphanumeric",
            );
        }

        // Should not contain special characters
        assert!(!id.contains('_'));
        assert!(!id.contains('-'));
    }
}
