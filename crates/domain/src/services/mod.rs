//! Domain services implementing business logic that spans entities.
//!
//! A service belongs here when a rule is too behavior-heavy for an entity or
//! a value object, yet is pure business logic — no I/O, no frameworks. Rules
//! that need persistence go through repository traits injected into the
//! service.
//!
//! The template ships no domain services yet — the `User` slice's rules all
//! fit inside its entities and value objects. Add one per `<concept>.rs`
//! file named `<Concept>Service`, re-exported below.
