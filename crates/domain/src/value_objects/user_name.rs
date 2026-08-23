//! The [`UserNameVo`] value object: a bounded, whitespace-trimmed display
//! name.

use crate::errors::ValueError;

/// Maximum character length accepted for a display name.
pub const USER_NAME_MAX_LEN: usize = 100;

/// A user's display name, validated at construction and immutable after.
///
/// Construct via [`UserNameVo::parse`]; never from bare strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserNameVo(String);

impl UserNameVo {
    /// Validates `raw` and wraps it in a [`UserNameVo`].
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Empty`] when `raw` is empty or only whitespace,
    /// [`ValueError::TooLong`] past [`USER_NAME_MAX_LEN`], and
    /// [`ValueError::Format`] when the name has leading or trailing
    /// whitespace (callers are expected to trim before parsing).
    pub fn parse(raw: &str) -> Result<Self, ValueError> {
        if raw.is_empty() {
            return Err(ValueError::Empty);
        }
        if raw.len() > USER_NAME_MAX_LEN {
            return Err(ValueError::TooLong {
                max: USER_NAME_MAX_LEN,
            });
        }
        if raw.trim() != raw {
            return Err(ValueError::Format {
                reason: "must not start or end with whitespace".into(),
            });
        }

        Ok(Self(raw.to_owned()))
    }

    /// Borrows the validated name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_plain_name() {
        let parsed = UserNameVo::parse("Ada Lovelace").expect("valid name");
        assert_eq!(parsed.as_str(), "Ada Lovelace");
    }

    #[test]
    fn rejects_empty_and_overlong_input() {
        assert!(matches!(UserNameVo::parse(""), Err(ValueError::Empty)));
        let raw = "a".repeat(USER_NAME_MAX_LEN + 1);
        assert!(matches!(
            UserNameVo::parse(&raw),
            Err(ValueError::TooLong { max: USER_NAME_MAX_LEN })
        ));
    }

    #[test]
    fn rejects_untrimmed_input() {
        assert!(matches!(
            UserNameVo::parse(" Ada"),
            Err(ValueError::Format { .. })
        ));
        assert!(matches!(
            UserNameVo::parse("Ada\n"),
            Err(ValueError::Format { .. })
        ));
    }
}
