//! Error types shared across the domain layer.
//!
//! These types are part of the domain's contracts: value objects return
//! [`ValueError`] when an invariant is violated, and repository ports return
//! [`UserRepositoryError`] when persistence fails. Adapters map their
//! technology-specific errors into these types at the boundary; raw driver
//! errors must never leak inward past this layer.

use thiserror::Error;

/// A value object invariant was violated during construction.
#[derive(Debug, Error)]
pub enum ValueError {
    /// The raw input was empty (or only whitespace).
    #[error("value must not be empty")]
    Empty,

    /// The raw input exceeded the value's maximum length.
    #[error("value exceeds its maximum length of {max}")]
    TooLong {
        /// The maximum length the value allows.
        max: usize,
    },

    /// The raw input was structurally malformed for this kind of value.
    #[error("invalid format: {reason}")]
    Format {
        /// A human-readable explanation of what made the input malformed.
        reason: String,
    },
}

/// Persisting or retrieving a user failed.
///
/// The use-case layer maps [`Conflict`] onto application-level semantics
/// (e.g. "email already registered"); everything else surfaces as a storage
/// failure to be reported as an infrastructure problem.
///
/// [`Conflict`]: UserRepositoryError::Conflict
#[derive(Debug, Error)]
pub enum UserRepositoryError {
    /// The write violated a uniqueness constraint (e.g. a duplicate email).
    ///
    /// Detected at the adapter boundary from the database's constraint
    /// violation; see `SeaOrmUserRepository` for the concrete mapping.
    #[error("the record conflicts with existing state")]
    Conflict,

    /// The storage backend failed for a reason the domain cannot classify.
    ///
    /// Carries the formatted driver error for logs and API error bodies;
    /// the original error type never crosses into the domain.
    #[error("storage failure: {0}")]
    Storage(String),
}
