//! Typed values that carry invariants and replace raw primitives.
//!
//! A value object is constructed through a `parse`-style constructor that
//! validates every invariant once; everywhere else in the codebase it can be
//! used without re-checking. Each file holds one type named `<Concept>Vo`.
//!
//! To add one: create `<concept>.rs`, define the type with a private inner
//! value, a validating constructor, and accessors — then re-export below.

/// An email address.
pub mod email;
/// A user's identifier as assigned by persistence.
pub mod user_id;
/// A human-readable display name for a user.
pub mod user_name;

pub use email::EmailVo;
pub use user_id::UserIdVo;
pub use user_name::UserNameVo;
