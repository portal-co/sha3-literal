# sha3-literal

Compile-time SHA-3 hash literals for Rust.

This crate provides procedural macros that compute SHA-3 hashes at compile time, allowing you to embed hash values directly into your binary without runtime computation.

## Features

- Compile-time SHA-3 hash computation
- Support for SHA3-256 and SHA3-512
- Multiple input formats: strings, byte strings, byte arrays, and more
- Both binary and hex output formats

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
sha3-literal = "0.1"
```

### Basic Examples

```rust
use sha3_literal::{sha3_literal, sha3_hex_literal};

// Get a byte array hash
let hash: [u8; 32] = sha3_literal!("hello world");

// Get a hex-encoded string hash
let hex_hash: &str = sha3_hex_literal!("hello world");
```

### SHA3-512 Support

```rust
use sha3_literal::{sha3_512_literal, sha3_512_hex_literal};

// Get a 64-byte hash
let hash: [u8; 64] = sha3_512_literal!("hello world");

// Get a hex-encoded SHA3-512 hash
let hex_hash: &str = sha3_512_hex_literal!("hello world");
```

### Supported Input Formats

The macros support various input formats:

```rust
use sha3_literal::sha3_literal;

// String literal
let hash = sha3_literal!("hello");

// Byte string literal  
let hash = sha3_literal!(b"hello");

// Byte literal
let hash = sha3_literal!(b'x');

// Integer (single byte)
let hash = sha3_literal!(42);

// Array of bytes
let hash = sha3_literal!([1, 2, 3, 4, 5]);

// Include external files
let hash = sha3_literal!(include_bytes!("path/to/file"));
let hash = sha3_literal!(include_str!("path/to/file.txt"));
```

## Architecture

This crate is built on top of `hash-literal-core`, a generic library for creating hash literal proc macros. The `sha3-literal` crate specializes this for SHA-3 hash algorithms.

## License

This project is licensed under CC0-1.0.

## Goals
- [ ] Add project goals

## Progress
- [ ] Initial setup

---
*AI assisted*
