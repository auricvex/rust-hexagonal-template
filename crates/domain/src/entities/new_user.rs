//! The un-persisted draft form of a [`crate::entities::UserEntity`].
//!
//! A draft has everything except the identifier, which only storage can
//! assign. Use cases build one from validated value objects and hand it to
//! [`UserRepository::insert`], receiving back the full entity.

use crate::value_objects::{EmailVo, UserNameVo};

/// A user awaiting its first persist — validated parts, no identity yet.
///
/// Construct via [`NewUserEntity::new`]; like the full entity, it cannot
/// exist in an invalid state.
#[derive(Debug, Clone)]
pub struct NewUserEntity {
    email: EmailVo,
    name: UserNameVo,
}

impl NewUserEntity {
    /// Assembles a draft from already-validated parts.
    pub fn new(email: EmailVo, name: UserNameVo) -> Self {
        Self { email, name }
    }

    /// The draft's validated email address.
    pub fn email(&self) -> &EmailVo {
        &self.email
    }

    /// The draft's validated display name.
    pub fn name(&self) -> &UserNameVo {
        &self.name
    }
}
