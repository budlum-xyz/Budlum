//! Shared, dependency-light primitives for Budlum workspace crates.
pub mod address;
pub mod hash;
pub use address::Address;
pub use hash::{calculate_hash, calculate_hash_bytes, hash_fields, hash_fields_bytes};
