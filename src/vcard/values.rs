//! vCard value types (RFC 6350 §4).
//!
//! Per the crate's rule enforcement, vCard properties hold only values from
//! here. The value types whose grammar is identical in vCard ([`Text`],
//! [`Uri`], [`MediaType`], [`Binary`]) are the crate's own, re-exposed. The
//! ones that only look alike are defined here instead: [`Boolean`] is
//! case-insensitive, [`Integer`] is 64-bit, [`Float`] forbids scientific
//! notation, [`UtcOffset`] allows a bare `+hh`, and the date and time types
//! carry vCard's reduced-accuracy and truncated forms.
//!
//! Escaping (RFC 6350 §3.4) is decoded by [`Text`] itself, which is right
//! for vCard too: `\\`, `\,`, `\;` and `\n`/`\N` are decoded, an unknown
//! escape is left alone, and an unescaped `,` or `;` in a single value is
//! accepted rather than rejected (real producers write them). Writing always
//! escapes them, which is valid everywhere and required inside a list or a
//! structured value.

mod composite;
mod datetime;
mod structured;

pub use crate::values::{Binary, MediaType, Text, Uri};
pub use composite::{DateAndOrTimeOrText, Kind, TextOrUri, Timezone, Uid};
pub use datetime::{
    Date, DateAndOrTime, DateTime, Time, Timestamp, UtcOffset, Zone,
};
pub use structured::{Address, ClientPidMap, Gender, Name, Organization, Sex};

use super::ParseError;
use crate::ast::split_unescaped;
use std::{fmt, str::Utf8Error};
use thiserror::Error;

/// A value that doesn't match the grammar of its value type.
#[derive(Debug, Error)]
pub enum ValueError {
    /// A [`Text`] or [`Uri`] inside the value failed to parse.
    #[error(transparent)]
    Text(#[from] crate::values::ValueError),

    /// The value isn't the shape its type needs.
    #[error("Malformed {expected}, got {received:?}")]
    Malformed {
        /// What the value is supposed to be.
        expected: &'static str,
        /// What was found there.
        received: String,
    },

    /// Text that isn't UTF-8.
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
}

/// Builds the [`ValueError::Malformed`] every value type shares.
fn malformed(expected: &'static str, received: &[u8]) -> ValueError {
    ValueError::Malformed {
        expected,
        received: String::from_utf8_lossy(received).into_owned(),
    }
}

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

/// A comma-separated list of values of one type.
///
/// A COMMA in a value of the list is escaped with a BACKSLASH, so it isn't
/// mistaken for a delimiter. The list has at least one element, which can
/// itself be empty.
///
/// ```text
/// text-list             = text             *("," text)
/// date-list             = date             *("," date)
/// time-list             = time             *("," time)
/// date-time-list        = date-time        *("," date-time)
/// date-and-or-time-list = date-and-or-time *("," date-and-or-time)
/// timestamp-list        = timestamp        *("," timestamp)
/// integer-list          = integer          *("," integer)
/// float-list            = float            *("," float)
/// ```
///
/// Example:
///
/// > this is one value,this is another
///
/// [Section 4](https://datatracker.ietf.org/doc/html/rfc6350#section-4)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List<T>(Vec<T>);

impl<T> List<T> {
    /// Builds a list from its elements.
    pub fn new(items: Vec<T>) -> Self {
        Self(items)
    }

    /// The elements, in order.
    pub fn items(&self) -> &[T] {
        &self.0
    }
}

impl<T> TryFrom<&[u8]> for List<T>
where
    T: for<'a> TryFrom<&'a [u8]>,
    for<'a> <T as TryFrom<&'a [u8]>>::Error: Into<ValueError>,
{
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        split_unescaped(v, b',')
            .into_iter()
            .map(|item| T::try_from(item).map_err(Into::into))
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

impl<T: fmt::Display> fmt::Display for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, item) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            item.fmt(f)?;
        }
        Ok(())
    }
}

/// A `text-list`, e.g. the value of `NICKNAME` or `CATEGORIES`.
pub type TextList = List<Text>;
/// A `date-list`.
pub type DateList = List<Date>;
/// A `time-list`.
pub type TimeList = List<Time>;
/// A `date-time-list`.
pub type DateTimeList = List<DateTime>;
/// A `date-and-or-time-list`.
pub type DateAndOrTimeList = List<DateAndOrTime>;
/// A `timestamp-list`.
pub type TimestampList = List<Timestamp>;
/// An `integer-list`.
pub type IntegerList = List<Integer>;
/// A `float-list`.
pub type FloatList = List<Float>;

/// The `boolean` value type is used to express boolean values. These values
/// are case-insensitive.
///
/// Example:
///
/// > TRUE
/// >
/// > false
/// >
/// > True
///
/// [Section 4.4](https://datatracker.ietf.org/doc/html/rfc6350#section-4.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Boolean(bool);

impl Boolean {
    /// Builds a `boolean` from a native `bool`.
    pub fn new(value: bool) -> Self {
        Self(value)
    }

    /// The value.
    pub fn value(&self) -> bool {
        self.0
    }
}

impl TryFrom<&[u8]> for Boolean {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        if v.eq_ignore_ascii_case(b"TRUE") {
            Ok(Self(true))
        } else if v.eq_ignore_ascii_case(b"FALSE") {
            Ok(Self(false))
        } else {
            Err(malformed("boolean (TRUE or FALSE)", v))
        }
    }
}

impl fmt::Display for Boolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if self.0 { "TRUE" } else { "FALSE" })
    }
}

/// The `integer` value type is used to express signed integers in decimal
/// format. If sign is not specified, the value is assumed positive "+".
/// Multiple "integer" values can be specified using the comma-separated
/// notation. The maximum value is 9223372036854775807, and the minimum value
/// is -9223372036854775808. These limits correspond to a signed 64-bit
/// integer using two's-complement arithmetic.
///
/// Example:
///
/// > 1234567890
/// >
/// > -1234556790
/// >
/// > +1234556790,432109876
///
/// [Section 4.5](https://datatracker.ietf.org/doc/html/rfc6350#section-4.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Integer(i64);

impl Integer {
    /// Builds an `integer` from a native `i64`.
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    /// The value.
    pub fn value(&self) -> i64 {
        self.0
    }
}

/// `[sign] 1*DIGIT`, the digits after the optional sign.
fn unsigned(v: &[u8]) -> &[u8] {
    match v.first() {
        Some(b'+' | b'-') => &v[1..],
        _ => v,
    }
}

impl TryFrom<&[u8]> for Integer {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let digits = unsigned(v);
        if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
            return Err(malformed("integer", v));
        }
        std::str::from_utf8(v)?
            .parse()
            .map(Self)
            .map_err(|_| malformed("integer within 64 bits", v))
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The `float` value type is used to express real numbers. If sign is not
/// specified, the value is assumed positive "+". Multiple "float" values can
/// be specified using the comma-separated notation. Implementations MUST
/// support a precision equal or better than that of the IEEE "binary64"
/// format.
///
/// Note: Scientific notation is disallowed. Implementers wishing to use
/// their favorite language's %f formatting should be careful.
///
/// Example:
///
/// > 20.30
/// >
/// > 1000000.0000001
/// >
/// > 1.333,3.14
///
/// [Section 4.6](https://datatracker.ietf.org/doc/html/rfc6350#section-4.6)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Float(f64);

impl Float {
    /// Builds a `float` from a native `f64`. NaN and the infinities have no
    /// representation in the grammar, so they're refused.
    pub fn new(value: f64) -> Result<Self, ValueError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(ValueError::Malformed {
                expected: "finite float",
                received: value.to_string(),
            })
        }
    }

    /// The value.
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl TryFrom<&[u8]> for Float {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        // `[sign] 1*DIGIT ["." 1*DIGIT]`: no exponent, no `inf`/`nan`, no
        // bare `.5` or `5.`, all of which `f64::from_str` would accept.
        let unsigned = unsigned(v);
        let (whole, fraction) = match unsigned.iter().position(|b| *b == b'.') {
            Some(dot) => (&unsigned[..dot], Some(&unsigned[dot + 1..])),
            None => (unsigned, None),
        };
        let digits =
            |d: &[u8]| !d.is_empty() && d.iter().all(u8::is_ascii_digit);
        if !digits(whole) || fraction.is_some_and(|f| !digits(f)) {
            return Err(malformed("float", v));
        }
        std::str::from_utf8(v)?
            .parse()
            .map(Self)
            .map_err(|_| malformed("float", v))
    }
}

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `f64`'s `Display` never switches to scientific notation.
        self.0.fmt(f)
    }
}

/// A single language tag, as defined in RFC 5646.
///
/// Example:
///
/// > en-US
/// >
/// > zh-Hant-TW
///
/// [Section 4.8](https://datatracker.ietf.org/doc/html/rfc6350#section-4.8)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageTag(langtag::LangTagBuf);

impl LanguageTag {
    /// The tag as written.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<&[u8]> for LanguageTag {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        langtag::LangTagBuf::from_bytes(v.to_vec())
            .map(Self)
            .map_err(|_| malformed("RFC 5646 language tag", v))
    }
}

impl fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A parameter value: `param-value = *SAFE-CHAR / DQUOTE *QSAFE-CHAR DQUOTE`.
///
/// Property parameter value elements that contain the COLON (U+003A),
/// SEMICOLON (U+003B), or COMMA (U+002C) character separators MUST be
/// specified as quoted-string text values. Property parameter values MUST
/// NOT contain the DQUOTE (U+0022) character. The DQUOTE character is used
/// as a delimiter for parameter values that contain restricted characters
/// or URI text.
///
/// The value is held decoded: the quotes are gone, and the RFC 6868
/// caret-encoding (`^n`, `^'`, `^^`) is undone, which is what lets a value
/// carry a newline or a DQUOTE. Writing quotes and encodes again as needed.
///
/// Example:
///
/// > LABEL="Mr. John Q. Public, Esq.^nNew York, NY 10001^nU.S.A."
///
/// [Section 5](https://datatracker.ietf.org/doc/html/rfc6350#section-5)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamValue(String);

impl ParamValue {
    /// Builds a value from its decoded text.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The decoded text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Decodes the content of one param-value, between its quotes if it had
    /// any.
    pub(crate) fn decode(inner: &[u8]) -> Result<Self, ValueError> {
        Ok(Self(
            std::str::from_utf8(&crate::ast::decode_caret(inner))?.to_owned(),
        ))
    }
}

impl TryFrom<&[u8]> for ParamValue {
    type Error = ValueError;

    /// A param-value as written, with its quotes if it has them.
    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let inner = crate::ast::strip_quoted_string(v).unwrap_or(v);
        if inner.contains(&b'"') {
            return Err(malformed("param-value without a stray DQUOTE", v));
        }
        Self::decode(inner)
    }
}

impl fmt::Display for ParamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let quoted = self.0.contains([',', ';', ':']);
        if quoted {
            f.write_str("\"")?;
        }
        for c in self.0.chars() {
            match c {
                '^' => f.write_str("^^")?,
                '"' => f.write_str("^'")?,
                '\n' => f.write_str("^n")?,
                '\r' => {}
                c => write!(f, "{c}")?,
            }
        }
        if quoted {
            f.write_str("\"")?;
        }
        Ok(())
    }
}
