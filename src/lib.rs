//! RFC 5545 in Rust
#![warn(missing_docs)]

pub(crate) mod ast;
pub use ast::ComponentError;
mod calendar;
pub use calendar::{Calendar, CalendarParseError, Component};
/// As specified [in the RFC Section 3.6](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6)
pub mod components;
/// Sections 3.7 and 3.8 of the RFC
pub mod properties;
mod rrule;
pub use rrule::*;
/// RFC 5546: the iCalendar Transport-Independent Interoperability Protocol
/// (iTIP), which defines the scheduling semantics layered on top of RFC
/// 5545's `METHOD` property.
#[cfg(feature = "rfc_5546")]
pub mod itip;

/// Alias for emails. TODO: enforce email safety
pub type Email = String;

/// A globally unique identifier for a calendar component.
///
/// Typically generated from the current timestamp and a random suffix so it is
/// unique across calendar stores.  See [`Uid::new`] for the canonical
/// constructor.
#[derive(Debug)]
pub struct Uid(String);

/// A pair of two values of the same type, used for properties such as [`Geo`]
/// that carry two coordinates.
#[derive(Debug)]
pub struct Pair<T>(T, T);

impl<T> Pair<T> {
    /// Constructs a pair directly from its two values.
    pub fn new(a: T, b: T) -> Self {
        Self(a, b)
    }
}

impl<T> TryFrom<&[u8]> for Pair<T>
where
    T: for<'a> TryFrom<&'a [u8], Error = values::ValueError>,
{
    type Error = values::ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let (a, b) =
            ast::split_once(v, b';').ok_or(values::ValueError::Malformed {
                expected: "first;second".into(),
                received: std::str::from_utf8(v).ok().map(Into::into),
            })?;
        Ok(Self(a.try_into()?, b.try_into()?))
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Pair<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};{}", self.0, self.1)
    }
}

/// A property can have attributes with which it is associated.  These
/// "property parameters" contain meta-information about the property or
/// the property value.  Property parameters are provided to specify such
/// information as the location of an alternate text representation for a
/// property value, the language of a text property value, the value type
/// of the property value, and other attributes.
///
/// Property parameter values that contain the COLON, SEMICOLON, or COMMA
/// character separators MUST be specified as quoted-string text values.
/// Property parameter values MUST NOT contain the DQUOTE character.  The
/// DQUOTE character is used as a delimiter for parameter values that
/// contain restricted characters or URI text.  For example:
///
/// > DESCRIPTION;ALTREP="cid:part1.0001@example.org":The Fall'98 Wild
/// > Wizards Conference - - Las Vegas\, NV\, USA
///
/// Property parameter values that are not in quoted-strings are
/// case-insensitive.
///
/// [More](https://datatracker.ietf.org/doc/html/rfc5545#autoid-11)
pub mod params;
/// The properties in an iCalendar object are strongly typed.  The
/// definition of each property restricts the value to be one of the
/// value data types, or simply value types, defined in this section.
/// The value type for a property will either be specified implicitly as
/// the default value type or will be explicitly specified with the
/// "VALUE" parameter.  If the value type of a property is one of the
/// alternate valid types, then it MUST be explicitly specified with the
/// "VALUE" parameter.
///
/// [More in the RFC](https://datatracker.ietf.org/doc/html/rfc5545#autoid-32)
pub mod values;
impl Uid {
    /// Creates a new UID
    ///
    /// Taks the local time as rfc3339 and salts with nanoid
    pub fn new() -> Self {
        let now = chrono::Local::now();
        Self(format!(
            "{}-{}@ogonek.app",
            now.to_rfc3339(),
            nanoid::nanoid!(5),
        ))
    }

    /// Gets the UID as reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Uid {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::Float;

    #[test]
    fn pair_parses_geo_example() {
        let pair = Pair::<Float>::try_from(b"37.386013;-122.082932".as_slice())
            .unwrap();
        assert_eq!(*pair.0, 37.386013);
        assert_eq!(*pair.1, -122.082932);
    }
}
