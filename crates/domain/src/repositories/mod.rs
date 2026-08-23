//! Repository interfaces for persisting and retrieving aggregates.
//!
//! Repositories are the domain's persistence ports (see AGENTS.md §6,
//! *Ports vs. repositories*): they speak collection language about domain
//! entities only — `insert`, `find_by_id`, … — and are implemented by
//! outgoing adapters, never by the inside. Every method traffics in domain
//! types (`…Entity`, `…Vo`); a signature mentioning DTOs or transport types
//! does not belong here.
//!
//! To add one: create `<concept>.rs`, define the `<Concept>Repository`
//! trait plus its `<Concept>RepositoryError` (in `crate::errors`), then
//! re-export below.

/// Persistence contract for the `User` aggregate.
pub mod user;

pub use user::UserRepository;
