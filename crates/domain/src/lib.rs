//! Domain layer: entities, value objects, and business rules.
//!
//! This crate is the innermost layer of the application and must not depend
//! on any other workspace crate. Its external dependencies are kept near
//! zero: `async-trait` (for object-safe ports) and `thiserror` (for error
//! derive) only — no frameworks, no I/O, no async runtime.
//!
//! The `User` aggregate is the template's reference vertical slice; follow
//! its seams when adding a new concept.

/// Core business entities that model the problem domain.
pub mod entities;
/// Error types shared across the domain layer.
pub mod errors;
/// Port interfaces for outward capabilities other than persistence.
pub mod ports;
/// Repository interfaces for persisting and retrieving aggregates.
pub mod repositories;
/// Domain services implementing business logic that spans entities.
pub mod services;
/// Typed values that carry invariants and replace raw primitives.
pub mod value_objects;

pub use entities::{NewUserEntity, UserEntity};
pub use errors::{UserRepositoryError, ValueError};
pub use repositories::UserRepository;
pub use value_objects::{EmailVo, UserIdVo, UserNameVo};
