//! Core business entities that model the problem domain.
//!
//! An entity is identified by its identity (not its attribute values), has a
//! lifecycle, and enforces its own invariants from construction onward. Each
//! file here holds one aggregate root named `<Concept>Entity`.
//!
//! To add an entity: create `<concept>.rs`, define the type with private
//! fields and a validating constructor, then re-export it below.

/// The `User` aggregate root and its un-persisted draft form.
pub mod new_user;
/// The `User` aggregate root.
pub mod user;

pub use new_user::NewUserEntity;
pub use user::UserEntity;
