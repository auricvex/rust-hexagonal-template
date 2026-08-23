//! The [`EmailVo`] value object: a syntactically valid, length-bounded email
//! address.
//!
//! Validation is deliberately dependency-free (no `regex`): it enforces the
//! structural invariants that matter for storage and uniqueness — non-empty,
//! bounded length, exactly one `@`, and a plausible domain part. It is not a
//! full RFC 5322 parser; deliverability is out of scope for the domain.

use crate::errors::ValueError;

/// Maximum byte length accepted for an email address (RFC 5321 limit).
pub const EMAIL_MAX_LEN: usize = 254;

/// An email address whose structural invariants were validated at
/// construction and hold for its entire lifetime.
///
/// Construct via [`EmailVo::parse`]; never through string literals. The
/// value is immutable, cheap to clone, and usable as a map key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmailVo(String);

impl EmailVo {
    /// Validates `raw` and wraps it in an [`EmailVo`].
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Empty`] when `raw` is empty or only whitespace,
    /// [`ValueError::TooLong`] past [`EMAIL_MAX_LEN`], and
    /// [`ValueError::Format`] when the `local@domain` shape is malformed
    /// (missing/duplicated `@`, empty local or domain part, domain without a
    /// dot, or embedded whitespace).
    pub fn parse(raw: &str) -> Result<Self, ValueError> {
        if raw.trim().is_empty() {
            return Err(ValueError::Empty);
        }
        if raw.len() > EMAIL_MAX_LEN {
            return Err(ValueError::TooLong { max: EMAIL_MAX_LEN });
        }

        let Some((local, domain)) = raw.split_once('@') else {
            return Err(ValueError::Format {
                reason: "must contain '@'".into(),
            });
        };
        // Exactly one '@': neither part may itself contain one.
        if local.contains('@') || domain.contains('@') {
            return Err(ValueError::Format {
                reason: "must contain exactly one '@'".into(),
            });
        }
        if local.is_empty() {
            return Err(ValueError::Format {
                reason: "local part must not be empty".into(),
            });
        }
        if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
            return Err(ValueError::Format {
                reason: "domain must contain a '.' separating non-empty labels".into(),
            });
        }

        Ok(Self(raw.to_owned()))
    }

    /// Borrows the validated address.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_well_formed_address() {
        let parsed = EmailVo::parse("ada@example.com").expect("valid email");
        assert_eq!(parsed.as_str(), "ada@example.com");
    }

    #[test]
    fn rejects_empty_and_blank_input() {
        assert!(matches!(EmailVo::parse(""), Err(ValueError::Empty)));
        assert!(matches!(EmailVo::parse("   "), Err(ValueError::Empty)));
    }

    #[test]
    fn rejects_overlong_input() {
        let local = "a".repeat(EMAIL_MAX_LEN);
        let raw = format!("{local}@example.com");
        assert!(matches!(EmailVo::parse(&raw), Err(ValueError::TooLong { max: EMAIL_MAX_LEN })));
    }

    #[test]
    fn rejects_malformed_shapes() {
        let cases = [
            "plainaddress",
            "a@b@c.com",
            "@example.com",
            "user@",
            "user@localhost",
            "user@.example.com",
            "user@example.com.",
            "us er@example.com",
        ];
        for raw in cases {
            assert!(
                matches!(EmailVo::parse(raw), Err(ValueError::Format { .. })),
                "expected Format error for {raw:?}"
            );
        }
    }
}
