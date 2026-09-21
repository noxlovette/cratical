//! vCard 4.0 ([RFC 6350](https://datatracker.ietf.org/doc/html/rfc6350))
//! and 3.0 ([RFC 2426](https://datatracker.ietf.org/doc/html/rfc2426)), the
//! payloads CardDAV ([RFC 6352](https://datatracker.ietf.org/doc/html/rfc6352))
//! carries.
//!
//! A model of its own, not part of the iCalendar one: property names such as
//! `UID`, `URL`, `CATEGORIES`, `SOURCE`, `NAME`, `TZ` and `GEO` exist in both
//! formats with different value types, so [`values`], [`params`] and
//! [`properties`] are this module's own, and properties hold only values and
//! parameters from them. Only server-side CardDAV (WebDAV, XML, queries,
//! filters) is out of scope.
//!
//! [`VCard::parse`] and [`VCard::parse_stream`] read a vCard's `VERSION` and
//! every property of RFC 6350 §6 as its own type in [`properties`], group
//! included; `X-` and unknown properties are kept whole. A card is checked as
//! a whole before it is handed back (see [`VCardBuilder`]), and a card built
//! from code, with [`VCard::builder`], goes through the same checks. Writing
//! a card with `Display` gives back what parses to the same card, folded at
//! 75 octets.

use crate::ast::{Lexer, LexerError};
use thiserror::Error;

mod builder;
pub mod params;
pub(crate) mod parser;
pub mod properties;
pub mod values;

pub use builder::VCardBuilder;
use properties::{FormattedName, Property, Uid, Version};
use std::fmt;

/// A vCard object: `BEGIN:VCARD`, its properties, `END:VCARD`.
///
/// Example:
///
/// > BEGIN:VCARD
/// > VERSION:4.0
/// > FN:Simon Perreault
/// > END:VCARD
///
/// [Section 6.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VCard {
    version: Version,
    properties: Vec<Property>,
}

impl VCard {
    /// The `VERSION` property.
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Every property but `VERSION`, in the order they were written.
    pub fn properties(&self) -> &[Property] {
        &self.properties
    }

    /// The `FN` properties, in order; there is at least one (§6.2.1).
    pub fn formatted_names(&self) -> impl Iterator<Item = &FormattedName> {
        self.properties.iter().filter_map(|p| match p {
            Property::FormattedName(f) => Some(f),
            _ => None,
        })
    }

    /// The kind of object the card represents: its `KIND`, or "individual"
    /// when it has none (§6.1.4).
    pub fn kind(&self) -> values::Kind {
        self.properties
            .iter()
            .find_map(|p| match p {
                Property::Kind(k) => Some(k.value().clone()),
                _ => None,
            })
            .unwrap_or(values::Kind::Individual)
    }

    /// The card's `UID`, if it has one. It is optional in RFC 6350, but a
    /// CardDAV address object must have one (RFC 6352 §5.1), and this is
    /// the cheap way to check.
    pub fn uid(&self) -> Option<&Uid> {
        self.properties.iter().find_map(|p| match p {
            Property::Uid(u) => Some(u),
            _ => None,
        })
    }

    /// A [`VCardBuilder`] for a vCard 4.0 named `formatted_name`.
    pub fn builder(formatted_name: FormattedName) -> VCardBuilder {
        VCardBuilder::new(formatted_name)
    }

    /// Parses exactly one `BEGIN:VCARD ... END:VCARD` object, erroring on
    /// anything after it. Use [`VCard::parse_stream`] for a `.vcf` holding
    /// several.
    ///
    /// [Section 3.3](https://datatracker.ietf.org/doc/html/rfc6350#section-3.3)
    pub fn parse(src: &[u8]) -> Result<Self, ParseError> {
        let mut parser = parser::Parser::new(Lexer::vcard(src).scan()?);
        let card = parser.vcard()?;
        if !parser.is_at_end()? {
            return Err(ParseError::TrailingData);
        }
        Ok(card)
    }

    /// Parses a `vcard-entity` (`1*vcard`): any number of back-to-back
    /// `BEGIN:VCARD ... END:VCARD` objects, in order.
    ///
    /// Example:
    ///
    /// > BEGIN:VCARD
    /// > VERSION:4.0
    /// > ...
    /// > END:VCARD
    /// > BEGIN:VCARD
    /// > VERSION:4.0
    /// > ...
    /// > END:VCARD
    ///
    /// [Section 3.3](https://datatracker.ietf.org/doc/html/rfc6350#section-3.3)
    pub fn parse_stream(src: &[u8]) -> Result<Vec<Self>, ParseError> {
        let mut parser = parser::Parser::new(Lexer::vcard(src).scan()?);
        let mut cards = Vec::new();
        while !parser.is_at_end()? {
            cards.push(parser.vcard()?);
        }
        Ok(cards)
    }
}

impl fmt::Display for VCard {
    /// Writes the card as RFC 6350 §3.3 has it: `BEGIN:VCARD`, `VERSION`,
    /// the properties in order, `END:VCARD`, each line folded at 75 octets
    /// (§3.2) and ended by CRLF. Parsing it gives back an equal card.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_line(f, "BEGIN:VCARD")?;
        write_line(f, &self.version.to_string())?;
        for property in &self.properties {
            write_line(f, &property.to_string())?;
        }
        write_line(f, "END:VCARD")
    }
}

fn write_line(f: &mut fmt::Formatter<'_>, line: &str) -> fmt::Result {
    crate::ast::write_folded(f, line)
}

/// A card that breaks a rule of RFC 6350 that needs the whole card to
/// tell, found by [`VCardBuilder::build`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// No `FN`: "This property is required to be present" (§6.2.1).
    #[error("The vCard has no FN property")]
    MissingFormattedName,

    /// More than one instance of a property whose cardinality is `*1`,
    /// instances sharing an `ALTID` counting as one (§5.4).
    #[error("The vCard has more than one {0} property")]
    TooMany(&'static str),

    /// A `MEMBER` on a card that isn't `KIND:group` (§6.6.5).
    #[error("MEMBER is only allowed when KIND is group")]
    MemberWithoutGroup,

    /// A `PID` whose source identifier has no `CLIENTPIDMAP` (§6.7.7).
    #[error("PID source identifier {0} has no CLIENTPIDMAP")]
    UnmappedPidSource(u32),
}

/// Error returned when vCard content fails to parse.
#[derive(Debug, Error)]
pub enum ParseError {
    /// The raw input couldn't be tokenized.
    #[error(transparent)]
    Lexer(#[from] LexerError),

    /// A token turned up where the grammar doesn't allow it.
    #[error("Unexpected token at line {line} ({lexeme}): {msg}")]
    UnexpectedToken {
        /// The (post-unfolding) content line.
        line: usize,
        /// What was expected there.
        msg: &'static str,
        /// What was found.
        lexeme: String,
    },

    /// The input ended before `END:VCARD`.
    #[error("Unexpected end of input")]
    UnexpectedEof,

    /// The object started with `BEGIN:` followed by something but `VCARD`.
    #[error("Expected BEGIN:VCARD at line {line}, got BEGIN:{name}")]
    NotVCard {
        /// The (post-unfolding) content line.
        line: usize,
        /// The component name that followed `BEGIN:`.
        name: String,
    },

    /// `END:` named something but `VCARD`.
    #[error("END:{name} at line {line} doesn't match BEGIN:VCARD")]
    MismatchedEnd {
        /// The (post-unfolding) content line.
        line: usize,
        /// The component name that followed `END:`.
        name: String,
    },

    /// A `BEGIN` inside a vCard. Nesting isn't allowed in vCard 4.0.
    #[error("Nested BEGIN at line {line}, which vCard doesn't allow here")]
    NestedComponent {
        /// The (post-unfolding) content line.
        line: usize,
    },

    /// A vCard with no `VERSION` property.
    #[error("The vCard has no VERSION property")]
    MissingVersion,

    /// A 4.0 vCard whose `VERSION` isn't right after `BEGIN:VCARD`.
    #[error("VERSION must come immediately after BEGIN:VCARD in vCard 4.0")]
    VersionNotFirst,

    /// More than one `VERSION` property.
    #[error("The vCard has more than one VERSION property")]
    DuplicateVersion,

    /// A `VERSION` this crate doesn't support (or, without the `rfc-2426`
    /// feature, `3.0`).
    #[error("Unsupported vCard version {0:?}")]
    UnsupportedVersion(String),

    /// Something after the one object [`VCard::parse`] expects.
    #[error("Unexpected content after END:VCARD")]
    TrailingData,

    /// A group with characters outside `ALPHA / DIGIT / "-"`.
    #[error("Malformed property group {0:?}")]
    Group(String),

    /// A content line or parameter didn't have the expected shape.
    #[error("Malformed content line. Expected {expected}, got {received:?}")]
    Malformed {
        /// What was expected there.
        expected: String,
        /// What was found.
        received: Option<String>,
    },

    /// A parameter that doesn't follow RFC 6350 §5.
    #[error(transparent)]
    Param(#[from] params::ParamError),

    /// A `VALUE` parameter naming a value type the property can't have,
    /// e.g. `FN;VALUE=uri`.
    #[error("The {property} property can't have VALUE={value}")]
    ValueType {
        /// The property.
        property: &'static str,
        /// The value type that was named.
        value: String,
    },

    /// A `TYPE` value that RFC 6350 §5.6 reserves for another property:
    /// `type-param-tel` "MUST NOT be used with a property other than TEL",
    /// nor `type-param-related` with one other than RELATED.
    #[error("TYPE={value} is only for {only}, not {property}")]
    TypeValue {
        /// The property.
        property: &'static str,
        /// The `TYPE` value.
        value: String,
        /// The one property that may have it.
        only: &'static str,
    },

    /// A parameter the RFC forbids on the property.
    #[error("The {param} parameter can't be applied to {property}")]
    ParamNotAllowed {
        /// The property.
        property: &'static str,
        /// The parameter.
        param: &'static str,
    },

    /// The properties parsed, but the card as a whole breaks a rule.
    #[error(transparent)]
    Invalid(#[from] ValidationError),

    /// A property value that doesn't follow RFC 6350 §4.
    #[error(transparent)]
    Value(#[from] values::ValueError),

    /// Text that isn't UTF-8.
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}

/// The offset of the `:` that ends a content line's parameters and starts
/// its value: the first one outside a quoted-string.
fn value_start(v: &[u8]) -> Result<usize, ParseError> {
    crate::ast::find_unquoted(v, b':').ok_or_else(|| ParseError::Malformed {
        expected: "':' introducing the property value".into(),
        received: std::str::from_utf8(v).ok().map(Into::into),
    })
}
