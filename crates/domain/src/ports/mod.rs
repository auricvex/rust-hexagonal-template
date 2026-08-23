//! Driven ports for outward capabilities other than persistence.
//!
//! This folder holds every contract the domain needs from the outside world
//! that is *not* aggregate storage — sending mail, publishing events, hashing
//! passwords, reading the clock, calling an external API. Persistence
//! contracts live separately in [`crate::repositories`]; the split is
//! topical convention, never a mechanism difference (see AGENTS.md §6).
//!
//! Traits here are named `<Capability>Port` (e.g. `MailSenderPort`) and
//! follow the same rules as repositories: owned by the domain, implemented
//! by outgoing adapters, injected at the composition root.
//!
//! The template ships no capability ports yet — the reference slice needs
//! none. Add one here (with a `//`-free doc comment explaining its
//! contract) the first time a use case needs an outward capability.
