//! Values that are one of several types, chosen by the `VALUE` parameter:
//! `KIND`, `TEL`/`RELATED`/`KEY`, `UID`, `TZ` and `BDAY`/`ANNIVERSARY`.
//!
//! Which variant a value is depends on the property's `VALUE` parameter, so
//! there is no `TryFrom<&[u8]>` for most of them: the property picks the
//! variant and builds it from the type's own parser.

use super::{DateAndOrTime, Text, Uri, UtcOffset, ValueError, malformed};
use std::fmt;

/// The kind of object a vCard represents.
///
/// The value may be one of the following:
///
/// "individual" for a vCard representing a single person or entity. This is
/// the default kind of vCard.
///
/// "group" for a vCard representing a group of persons or entities. The
/// group's member entities can be other vCards or other types of entities,
/// such as email addresses or web sites.
///
/// "org" for a vCard representing an organization. An organization vCard
/// will not (in fact, MUST NOT) contain MEMBER properties.
///
/// "location" for a named geographical place.
///
/// An x-name. vCards MAY include private or experimental values for KIND.
///
/// An iana-token. Additional values may be registered with IANA.
///
/// Implementations MUST support the specific string values defined above.
/// The names are case-insensitive, and an `x-name` or `iana-token` this
/// crate doesn't know is kept as written in [`Kind::Other`].
///
/// Example:
///
/// > org
///
/// [Section 6.1.4](https://datatracker.ietf.org/doc/html/rfc6350#section-6.1.4)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// `individual`, the default.
    Individual,
    /// `group`.
    Group,
    /// `org`.
    Org,
    /// `location`.
    Location,
    /// An `x-name` or `iana-token`, as written.
    Other(String),
}

impl TryFrom<&[u8]> for Kind {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        if v.is_empty()
            || !v.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'-')
        {
            return Err(malformed("an iana-token or x-name", v));
        }
        let token = std::str::from_utf8(v)?;
        Ok(match token.to_ascii_lowercase().as_str() {
            "individual" => Self::Individual,
            "group" => Self::Group,
            "org" => Self::Org,
            "location" => Self::Location,
            _ => Self::Other(token.to_owned()),
        })
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Individual => "individual",
            Self::Group => "group",
            Self::Org => "org",
            Self::Location => "location",
            Self::Other(token) => token,
        })
    }
}

/// A value that is free-form text or a URI.
///
/// `TEL` is text by default (for backward compatibility with vCard 3) but
/// SHOULD be reset to a URI value with `VALUE=uri`; `RELATED` and `KEY` are
/// a URI by default and can be reset to text with `VALUE=text`.
///
/// Example:
///
/// > tel:+1-555-555-5555;ext=5555
///
/// [Section 6.4.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.4.1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextOrUri {
    /// Free-form text.
    Text(Text),
    /// A URI.
    Uri(Uri),
}

impl fmt::Display for TextOrUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(v) => write!(f, "{v}"),
            Self::Uri(v) => write!(f, "{v}"),
        }
    }
}

/// A value that identifies the entity the vCard is about.
///
/// A single URI value. It MAY also be reset to free-form text. The "uuid"
/// URN namespace defined in \[RFC4122\] is particularly well suited to this
/// task, but other URI schemes MAY be used. Free-form text MAY also be used.
///
/// Real-world data is full of `UID:1234-ABCD`, which is text, not a URI, and
/// CardDAV keys everything on the UID, so a card carrying one isn't
/// rejected. Without `VALUE=text` the value is a [`Uid::Uri`] when it parses
/// as one and a [`Uid::Text`] when it doesn't. `VALUE=text` always gives
/// text, and `VALUE=uri` insists on a URI.
///
/// Example:
///
/// > urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6
///
/// [Section 6.7.6](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.6)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Uid {
    /// A URI, as the RFC has it.
    Uri(Uri),
    /// Free-form text.
    Text(Text),
}

impl TryFrom<&[u8]> for Uid {
    type Error = ValueError;

    /// A URI when the value parses as one, text when it doesn't.
    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        Ok(match Uri::try_from(v) {
            Ok(uri) => Self::Uri(uri),
            Err(_) => Self::Text(Text::try_from(v)?),
        })
    }
}

impl fmt::Display for Uid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uri(v) => write!(f, "{v}"),
            Self::Text(v) => write!(f, "{v}"),
        }
    }
}

/// The time zone of the object the vCard represents.
///
/// The default is a single text value. It can also be reset to a single URI
/// or utc-offset value.
///
/// It is expected that names from the public-domain Olson database \[TZ-DB\]
/// will be used, but this is not a restriction.
///
/// Note that utc-offset values SHOULD NOT be used because the UTC offset
/// varies with time -- not just because of the usual daylight saving time
/// shifts that occur in may regions, but often entire regions will "re-base"
/// their overall offset. The actual offset may be +/- 1 hour (or perhaps a
/// little more) than the one given.
///
/// Example:
///
/// > Raleigh/North America
///
/// [Section 6.5.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.5.1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Timezone {
    /// A time zone name.
    Text(Text),
    /// A URI.
    Uri(Uri),
    /// A UTC offset.
    UtcOffset(UtcOffset),
}

impl fmt::Display for Timezone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(v) => write!(f, "{v}"),
            Self::Uri(v) => write!(f, "{v}"),
            Self::UtcOffset(v) => write!(f, "{v}"),
        }
    }
}

/// A date, or a time, or both, or free-form text.
///
/// The default is a single date-and-or-time value. It can also be reset to a
/// single text value with `VALUE=text` (`BDAY;VALUE=text:circa 1800`).
///
/// Example:
///
/// > --0415
///
/// [Section 6.2.5](https://datatracker.ietf.org/doc/html/rfc6350#section-6.2.5)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateAndOrTimeOrText {
    /// A date-and-or-time value.
    DateAndOrTime(DateAndOrTime),
    /// Free-form text.
    Text(Text),
}

impl fmt::Display for DateAndOrTimeOrText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DateAndOrTime(v) => write!(f, "{v}"),
            Self::Text(v) => write!(f, "{v}"),
        }
    }
}
