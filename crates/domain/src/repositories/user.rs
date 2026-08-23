//! The [`UserRepository`] port: persistence operations for users.

use async_trait::async_trait;

use crate::entities::{NewUserEntity, UserEntity};
use crate::errors::UserRepositoryError;
use crate::value_objects::{EmailVo, UserIdVo};

/// Persistence contract for [`UserEntity`] aggregates.
///
/// Implemented by outgoing adapters (e.g. `SeaOrmUserRepository`) and
/// injected into use cases at the composition root. Signatures traffic in
/// domain types only; adapters translate to and from their storage models.
///
/// Object-safe by design (`async_trait`): use cases hold
/// `Arc<dyn UserRepository>` so the composition root can swap adapters —
/// including in-memory fakes in tests — without touching the inside.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Persists a new user and returns the stored aggregate, complete with
    /// its backend-assigned [`UserIdVo`].
    ///
    /// # Errors
    ///
    /// Returns [`UserRepositoryError::Conflict`] when a uniqueness
    /// constraint rejects the insert (the duplicate-email case), and
    /// [`UserRepositoryError::Storage`] for any other backend failure.
    /// Callers that must reject duplicates up front check with
    /// [`find_by_email`](Self::find_by_email) first; `Conflict` then covers
    /// only the rare concurrent-insert race.
    async fn insert(&self, user: NewUserEntity) -> Result<UserEntity, UserRepositoryError>;

    /// Looks up a user by email address; `None` when no such user exists.
    ///
    /// # Errors
    ///
    /// Returns [`UserRepositoryError::Storage`] when the backend fails;
    /// "not found" is expressed as `Ok(None)`, never as an error.
    async fn find_by_email(
        &self,
        email: &EmailVo,
    ) -> Result<Option<UserEntity>, UserRepositoryError>;

    /// Looks up a user by identifier; `None` when no such user exists.
    ///
    /// # Errors
    ///
    /// Returns [`UserRepositoryError::Storage`] when the backend fails;
    /// "not found" is expressed as `Ok(None)`, never as an error.
    async fn find_by_id(&self, id: UserIdVo)
        -> Result<Option<UserEntity>, UserRepositoryError>;
}
