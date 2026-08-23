//! Application layer: use cases that orchestrate the domain model.
//!
//! Depends on the `domain` crate, and never on infrastructure crates.

/// Data transfer objects exchanged between the application and its adapters.
pub mod dtos;
/// Application use cases that orchestrate the domain model.
pub mod use_cases;
