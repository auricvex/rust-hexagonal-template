//! The [`UserEntity`] aggregate root and its construction seam.

use crate::value_objects::{EmailVo, UserIdVo, UserNameVo};

/// A registered user — the `User` aggregate root.
///
/// All fields are private and were validated before construction, so every
/// `UserEntity` in circulation satisfies the aggregate's invariants. Code
/// outside this crate can read state through accessors but can never build
/// or mutate an instance except through [`UserEntity::new`].
#[derive(Debug, Clone)]
pub struct UserEntity {
    id: UserIdVo,
    email: EmailVo,
    name: UserNameVo,
}

impl UserEntity {
    /// Assembles a user from already-validated parts.
    ///
    /// Infallible by design: invariant enforcement lives in the value
    /// objects, so a `UserEntity` cannot exist in an invalid state.
    pub fn new(id: UserIdVo, email: EmailVo, name: UserNameVo) -> Self {
        Self { id, email, name }
    }

    /// This user's persistence-assigned identifier.
    pub fn id(&self) -> UserIdVo {
        self.id
    }

    /// This user's validated email address.
    pub fn email(&self) -> &EmailVo {
        &self.email
    }

    /// This user's validated display name.
    pub fn name(&self) -> &UserNameVo {
        &self.name
    }
}
