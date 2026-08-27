# Identifiers Library

A flexible ID generation library for Rust applications supporting ULID, UUID v7, and NanoID formats with compile-time feature selection.

## Features

This library provides a unified interface for generating unique identifiers with three different backend implementations:

- **ULID** (default): Universally Unique Lexicographically Sortable Identifier
- **UUID v7**: Time-ordered UUID variant with Unix timestamp precision
- **NanoID**: Compact, URL-safe unique string ID generator

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
identifiers = { path = "../identifiers" }  # For workspace members
```

## Feature Flags

Select your preferred ID format via feature flags:

```toml
# Use ULID (default)
identifiers = { path = "../identifiers" }

# Use UUID v7
identifiers = { path = "../identifiers", default-features = false, features = ["uuid"] }

# Use NanoID
identifiers = { path = "../identifiers", default-features = false, features = ["nanoid"] }
```

## Usage

The library exports a consistent API regardless of the backend:

```rust
use identifiers::{generate, Id};

fn main() {
    // Generate a new ID
    let id: Id = generate();
    println!("Generated ID: {}", id);
}
```

## ID Format Comparison

| Format | Length | Example | Use Case |
|--------|--------|---------|----------|
| ULID | 26 chars | `01ARZ3NDEKTSV4RRFFQ69G5FAV` | Time-sortable, millisecond precision |
| UUID v7 | 36 chars | `018b4d2e-7b74-7c58-abcd-123456789abc` | Standard UUID format, time-ordered |
| NanoID | 21 chars | `V1StGXR8Z5jdHi6BJhvEi` | Compact, URL-safe, high entropy |

## Implementation Details

### ULID (Default)
- 128-bit compatibility with UUID
- Lexicographically sortable
- Canonical base32 encoding
- Millisecond timestamp precision

### UUID v7
- RFC 4122 compliant
- Unix timestamp-based
- Sortable by creation time
- Standard hyphenated format

### NanoID
- Custom alphabet (alphanumeric only)
- 21 character length for ~126 bits of entropy
- No special characters (URL-safe)
- Non-sortable, purely random

## Priority System

When multiple features are enabled, the library follows this priority:
1. ULID (highest priority)
2. UUID
3. NanoID (lowest priority)

## Testing

Run tests with different feature combinations:

```bash
# Test default (ULID)
cargo test -p identifiers

# Test UUID
cargo test -p identifiers --no-default-features --features uuid

# Test NanoID
cargo test -p identifiers --no-default-features --features nanoid

# Test all features
cargo test -p identifiers --all-features
```

## Performance Considerations

- **ULID**: Best for time-series data and distributed systems requiring sortability
- **UUID v7**: Best for compatibility with existing UUID infrastructure
- **NanoID**: Best for user-facing IDs where compactness matters

## Security

All ID generators use cryptographically secure random number generation via the `getrandom` crate, ensuring unpredictability and uniqueness.

## License

See the project's main LICENSE file.