//! vCard property parameters.
//!
//! Per the crate's rule enforcement, vCard properties hold only parameters
//! from here.

use super::ParseError;
use crate::properties::{param_name, param_segments};
use std::fmt;

/// The `*(";" param)` parameter list of a vCard content line.
///
/// Parameters are kept as the raw `NAME=VALUE` segments they were written as
/// (unquoted, caret-encoded and multi-valued forms untouched), so they
/// round-trip exactly.
///
/// A property parameter's name is case-insensitive and its value may
/// contain the COMMA, SEMICOLON and COLON separators only inside a
/// quoted-string.
///
/// Example:
///
/// > TEL;TYPE="work,voice";PREF=1:+1-555-555-5555
///
/// [Section 5](https://datatracker.ietf.org/doc/html/rfc6350#section-5)
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Parameters {
    segments: Vec<String>,
}

impl Parameters {
    /// Whether the property has no parameters.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

impl TryFrom<&[u8]> for Parameters {
    type Error = ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut segments = Vec::new();
        for segment in param_segments(v) {
            param_name(segment)?;
            segments.push(std::str::from_utf8(segment)?.to_owned());
        }
        Ok(Self { segments })
    }
}

impl fmt::Display for Parameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for segment in &self.segments {
            write!(f, ";{segment}")?;
        }
        Ok(())
    }
}
