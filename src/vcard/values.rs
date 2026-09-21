//! vCard value types.
//!
//! Per the crate's rule enforcement, vCard properties hold only values from
//! here. Only the vCard-specific types needed so far exist; the rest of
//! RFC 6350 §4 arrives with the typed properties.

use super::ParseError;
use std::fmt;

/// The version of the vCard specification a vCard object is formatted
/// according to.
///
/// vCard 4.0 ([RFC 6350](https://datatracker.ietf.org/doc/html/rfc6350)):
/// the value MUST be "4.0" if the vCard corresponds to this specification.
///
/// vCard 3.0 ([RFC 2426](https://datatracker.ietf.org/doc/html/rfc2426),
/// behind the `rfc-2426` feature): the value MUST be "3.0" if the vCard
/// corresponds to this specification.
///
/// Any other value, including vCard 2.1's "2.1", isn't supported.
///
/// Example:
///
/// > VERSION:4.0
///
/// [Section 6.7.9](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.9)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    /// `4.0`, RFC 6350.
    V4_0,
    /// `3.0`, RFC 2426.
    #[cfg(feature = "rfc-2426")]
    V3_0,
}

impl TryFrom<&[u8]> for Version {
    type Error = ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        match v {
            b"4.0" => Ok(Self::V4_0),
            #[cfg(feature = "rfc-2426")]
            b"3.0" => Ok(Self::V3_0),
            other => Err(ParseError::UnsupportedVersion(
                String::from_utf8_lossy(other).into_owned(),
            )),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::V4_0 => "4.0",
            #[cfg(feature = "rfc-2426")]
            Self::V3_0 => "3.0",
        })
    }
}

/// The name or value of an extension property, kept exactly as written
/// (no unescaping) so it round-trips untouched.
///
/// The properties and parameters defined by RFC 6350 can be extended.
/// Non-standard, private properties and parameters with a name starting
/// with "X-" may be defined bilaterally between two cooperating agents
/// without outside registration or standardization.
///
/// Example:
///
/// > X-FAVOURITE-COLOR:green
///
/// [Section 6.10](https://datatracker.ietf.org/doc/html/rfc6350#section-6.10)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raw(String);

impl Raw {
    /// The value as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&[u8]> for Raw {
    type Error = ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(std::str::from_utf8(v)?.to_owned()))
    }
}

impl fmt::Display for Raw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
