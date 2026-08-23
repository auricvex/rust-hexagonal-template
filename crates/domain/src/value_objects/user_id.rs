//! The [`UserIdVo`] value object: the persistence-assigned identifier of a
//! user.

use crate::errors::ValueError;

/// A user's identifier, assigned by the storage backend on first insert.
///
/// Wrapping the raw integer keeps function signatures honest about *which*
/// id they mean and gives ids a single validation point (positivity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserIdVo(i64);

impl UserIdVo {
    /// Validates `raw` as a usable identifier.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Format`] when `raw` is zero or negative —
    /// identifiers are assigned by the database starting at 1, so such a
    /// value can never reference an existing user.
    pub fn new(raw: i64) -> Result<Self, ValueError> {
        if raw <= 0 {
            return Err(ValueError::Format {
                reason: "identifier must be positive".into(),
            });
        }

        Ok(Self(raw))
    }

    /// The raw identifier value, suitable for database queries.
    pub fn get(&self) -> i64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_positive_ids() {
        assert_eq!(UserIdVo::new(1).expect("valid id").get(), 1);
        assert_eq!(UserIdVo::new(i64::MAX).expect("valid id").get(), i64::MAX);
    }

    #[test]
    fn rejects_zero_and_negative_ids() {
        for raw in [0, -1, i64::MIN] {
            assert!(
                matches!(UserIdVo::new(raw), Err(ValueError::Format { .. })),
                "expected Format error for {raw}"
            );
        }
    }
}
