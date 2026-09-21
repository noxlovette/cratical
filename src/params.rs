pub use crate::values::Recur;
use crate::{
    ast::strip_quoted_string,
    values::{Boolean, CalendarUserAddress, MediaType, Text, Uri, ValueError},
};
use chrono_tz::Tz;
use std::fmt::Debug;
use thiserror::Error;

/// [`strip_quoted_string`], erroring with [`ParamError::QuotedString`] if
/// `v` isn't a quoted-string.
fn quoted(v: &[u8]) -> Result<&[u8], ParamError> {
    strip_quoted_string(v).ok_or(ParamError::QuotedString)
}

/// For a parameter whose grammar is `param-value = paramtext /
/// quoted-string` (quoting is optional — unlike e.g. `ALTREP`/`DIR`, whose
/// RFC text explicitly requires a quoted-string): strips the DQUOTE
/// delimiters if `v` is quoted, otherwise passes it through unchanged.
fn maybe_quoted(v: &[u8]) -> &[u8] {
    strip_quoted_string(v).unwrap_or(v)
}

/// Explicit value type for a property, as carried by the `VALUE` parameter.
///
/// This parameter specifies the value type and format of the property value.
/// The property values MUST be of a single value type.  For example, on the
/// "DTSTART" property the value type defaults to DATE-TIME.  However, if
/// the value type is set to DATE, then the value MUST be a DATE value type.
///
/// Example:
///
/// > DTSTART;VALUE=DATE:19980101
///
/// [Section 3.2.20](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.20)
#[derive(Default, Debug)]
pub enum ValueDataType {
    /// Inline binary data encoded as BASE64.
    Binary,
    /// A URI reference (RFC 3986).
    Uri,
    #[default]
    /// Plain text, with BACKSLASH escaping for special characters.
    Text,
    /// `TRUE` or `FALSE`.
    Boolean,
    /// A calendar user address (`mailto:` URI).
    CalAddress,
    /// An ISO 8601 calendar date.
    Date,
    /// An ISO 8601 calendar date and time of day.
    DateTime,
    /// An ISO 8601 duration.
    Duration,
    /// A floating-point number.
    Float,
    /// A signed 32-bit integer.
    Integer,
    /// A time period (start/end or start/duration).
    Period,
    /// A recurrence rule (`RRULE`).
    Recur,
    /// An ISO 8601 time of day.
    Time,
    /// A UTC offset (e.g. `-0500`).
    UtcOffset,
    /// A non-standard `X-` prefixed type name.
    XName(Text),
    /// An IANA-registered type name.
    Iana(Text),
}

impl TryFrom<&[u8]> for ValueDataType {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let r = match b {
            b"BINARY" => Self::Binary,
            b"URI" => Self::Uri,
            b"TEXT" => Self::Text,
            b"BOOLEAN" => Self::Boolean,
            b"CAL-ADDRESS" => Self::CalAddress,
            b"DATE" => Self::Date,
            b"DATE-TIME" => Self::DateTime,
            b"DURATION" => Self::Duration,
            b"FLOAT" => Self::Float,
            b"INTEGER" => Self::Integer,
            b"PERIOD" => Self::Period,
            b"RECUR" => Self::Recur,
            b"TIME" => Self::Time,
            b"UTC-OFFSET" => Self::UtcOffset,
            x => {
                if x.to_ascii_uppercase().starts_with(b"X-") {
                    Self::XName(x.try_into()?)
                } else {
                    Self::Iana(x.try_into()?)
                }
            }
        };
        Ok(r)
    }
}

/// This parameter specifies a URI that points to an
/// alternate representation for a textual property value.  A property
/// specifying this parameter MUST also include a value that reflects
/// the default representation of the text value.  The URI parameter
/// value MUST be specified in a quoted-string.
///
/// > Note: While there is no restriction imposed on the URI schemes
/// > allowed for this parameter, Content Identifier (CID) [RFC2392],
/// > HTTP [RFC2616], and HTTPS [RFC2818] are the URI schemes most
/// > commonly used by current implementations.
///
/// [Section 3.2.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.1)
#[derive(Debug)]
pub struct Altrep(Uri);

impl TryFrom<&[u8]> for Altrep {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(quoted(b)?.try_into()?))
    }
}

impl Altrep {
    /// Builds an `ALTREP` parameter directly from an already-parsed
    /// [`Uri`], skipping the quoted-string text round-trip
    /// `TryFrom<&[u8]>` requires.
    pub fn new(uri: Uri) -> Self {
        Self(uri)
    }
}

/// This parameter can be specified on properties with a
/// CAL-ADDRESS value type.  The parameter specifies the common name
/// to be associated with the calendar user specified by the property.
/// The parameter value is text.  The parameter value can be used for
/// display text to be associated with the calendar address specified
/// by the property.
///
/// Example:
///
/// > ORGANIZER;CN=John Smith:mailto:jsmith@example.com
///
/// [Section 3.2.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.2)
#[derive(Debug)]
pub struct CommonName(Text);

impl TryFrom<&[u8]> for CommonName {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(maybe_quoted(b).try_into()?))
    }
}

impl CommonName {
    /// Builds a `CN` parameter directly from already-typed [`Text`],
    /// skipping the text round-trip `TryFrom<&[u8]>` requires.
    pub fn new(name: Text) -> Self {
        Self(name)
    }
}

/// This parameter can be specified on properties with a
/// CAL-ADDRESS value type.  This parameter specifies those calendar
/// users that have delegated their participation in a group-scheduled
/// event or to-do to the calendar user specified by the property.
/// The individual calendar address parameter values MUST each be
/// specified in a quoted-string.
///
/// Example:
///
/// > ATTENDEE;DELEGATED-FROM="mailto:jsmith@example.com":mailto:jdoe@example.
/// > com
///
/// [Section 3.2.4](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.4)
#[derive(Debug)]
pub struct Delegators(Vec<CalendarUserAddress>);

impl TryFrom<&[u8]> for Delegators {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let mut vec = Vec::new();
        for s in b.split(|b| *b == b',') {
            vec.push(quoted(s)?.try_into()?);
        }
        Ok(Self(vec))
    }
}

impl Delegators {
    /// Builds a `DELEGATED-FROM` parameter directly from already-typed
    /// calendar user addresses, skipping the quoted-string-list text
    /// round-trip `TryFrom<&[u8]>` requires.
    pub fn new(addresses: Vec<CalendarUserAddress>) -> Self {
        Self(addresses)
    }
}
/// This parameter can be specified on properties with a
/// CAL-ADDRESS value type.  This parameter specifies those calendar
/// users whom have been delegated participation in a group-scheduled
/// event or to-do by the calendar user specified by the property.
/// The individual calendar address parameter values MUST each be
/// specified in a quoted-string.
///
/// Example:
///
/// > ATTENDEE;DELEGATED-TO="mailto:jdoe@example.com","mailto:jqpublic
/// > @example.com":mailto:jsmith@example.com
///
/// [Section 3.2.5](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.5)
#[derive(Debug)]
pub struct Delegatees(Vec<CalendarUserAddress>);

impl TryFrom<&[u8]> for Delegatees {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let mut vec = Vec::new();
        for s in b.split(|b| *b == b',') {
            vec.push(quoted(s)?.try_into()?);
        }
        Ok(Self(vec))
    }
}

impl Delegatees {
    /// Builds a `DELEGATED-TO` parameter directly from already-typed
    /// calendar user addresses, skipping the quoted-string-list text
    /// round-trip `TryFrom<&[u8]>` requires.
    pub fn new(addresses: Vec<CalendarUserAddress>) -> Self {
        Self(addresses)
    }
}

/// This parameter specifies a reference to a directory entry associated with
/// the calendar user specified by the property.  The parameter value is a
/// URI.  The URI parameter value MUST be specified in a quoted-string.
///
/// Example:
///
/// > ORGANIZER;DIR="ldap://example.com:6666/o=ABC%20Industries,c=US???(cn=Jim%
/// > 20Dolittle)":mailto:jimdo@example.com
///
/// [Section 3.2.6](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.6)
#[derive(Debug)]
pub struct DirectoryEntryReference(Uri);

impl TryFrom<&[u8]> for DirectoryEntryReference {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(quoted(b)?.try_into()?))
    }
}

impl DirectoryEntryReference {
    /// Builds a `DIR` parameter directly from an already-parsed [`Uri`],
    /// skipping the quoted-string text round-trip `TryFrom<&[u8]>`
    /// requires.
    pub fn new(uri: Uri) -> Self {
        Self(uri)
    }
}

/// This property parameter identifies the inline encoding
/// used in a property value.  The default encoding is "8BIT",
/// corresponding to a property value consisting of text.  The
/// "BASE64" encoding type corresponds to a property value encoded
/// using the "BASE64" encoding defined in [RFC2045](https://datatracker.ietf.org/doc/html/rfc2045).
/// If the value type parameter is ";VALUE=BINARY", then the inline
/// encoding parameter MUST be specified with the value
/// ";ENCODING=BASE64".
///
/// [Section 3.2.7](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.7)
#[derive(Debug, Default)]
pub enum Encoding {
    /// Default 8-bit text encoding.
    Bit8,
    /// BASE64 binary encoding, required when `VALUE=BINARY`.
    #[default]
    Base64,
}

impl TryFrom<&[u8]> for Encoding {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        match b {
            b"BASE64" => Ok(Self::Base64),
            b"8BIT" => Ok(Self::Bit8),
            _ => Err(ParamError::Malformed {
                expected: "BASE64 or 8BIT".into(),
                received: std::str::from_utf8(b).ok().map(|s| s.into()),
            }),
        }
    }
}

/// This parameter can be specified on properties that are
/// used to reference an object.  The parameter specifies the media
/// type [RFC4288] of the referenced object.  For example, on the
/// "ATTACH" property, an FTP type URI value does not, by itself,
/// necessarily convey the type of content associated with the
/// resource.  The parameter value MUST be the text for either an
/// IANA-registered media type or a non-standard media type.
///
/// Example:
///
/// > ATTACH;FMTTYPE=application/msword:ftp://example.com/pub/docs/agenda.doc
///
/// TODO: replace with MIME
///
/// [Section 3.2.8](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.8)
#[derive(Default, Debug)]
pub struct Fmttype(MediaType);

impl TryFrom<&[u8]> for Fmttype {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(b.try_into()?))
    }
}

impl Fmttype {
    /// Builds a `FMTTYPE` parameter directly from an already-parsed
    /// [`MediaType`], skipping the `"/"`-split text round-trip
    /// `TryFrom<&[u8]>` requires.
    pub fn new(media_type: MediaType) -> Self {
        Self(media_type)
    }
}

/// This parameter specifies the free or busy time type.
/// The value FREE indicates that the time interval is free for
/// scheduling.  The value BUSY indicates that the time interval is
/// busy because one or more events have been scheduled for that
/// interval.  The value BUSY-UNAVAILABLE indicates that the time
/// interval is busy and that the interval can not be scheduled.  The
/// value BUSY-TENTATIVE indicates that the time interval is busy
/// because one or more events have been tentatively scheduled for
/// that interval.  If not specified on a property that allows this
/// parameter, the default is BUSY.  Applications MUST treat x-name
/// and iana-token values they don't recognize the same way as they
/// would the BUSY value.
///
/// Example:
///
/// > FREEBUSY;FBTYPE=BUSY:19980415T133000Z/19980415T170000Z
///
/// [Section 3.2.9](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.9)
#[derive(Default, Debug)]
pub enum Fbtype {
    /// The interval is free for scheduling.
    Free,
    /// The interval is busy (one or more events scheduled).
    #[default]
    Busy,
    /// The interval is busy and cannot be scheduled.
    BusyUnavailable,
    /// The interval is tentatively busy.
    BusyTentative,
    /// A non-standard, `X-`-prefixed value.
    X(Text),
    /// A value registered with IANA that isn't one of the values above.
    Iana(Text),
}

impl TryFrom<&[u8]> for Fbtype {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let r = match b {
            b"FREE" => Self::Free,
            b"BUSY" => Self::Busy,
            b"BUSY-UNAVAILABLE" => Self::BusyUnavailable,
            b"BUSY-TENTATIVE" => Self::BusyTentative,
            x => {
                if x.to_ascii_uppercase().starts_with(b"X-") {
                    Self::X(x.try_into()?)
                } else {
                    Self::Iana(x.try_into()?)
                }
            }
        };
        Ok(r)
    }
}

/// This parameter identifies the language of the text in
/// the property value and of all property parameter values of the
/// property.  The value of the "LANGUAGE" property parameter is that
/// defined in [RFC5646].
///
/// For transport in a MIME entity, the Content-Language header field
/// can be used to set the default language for the entire body part.
/// Otherwise, no default language is assumed.
///
/// The following are examples of this parameter on the
/// "SUMMARY" and "LOCATION" properties:
///
/// > SUMMARY;LANGUAGE=en-US:Company Holiday Party
/// >
/// > LOCATION;LANGUAGE=en:Germany
/// >
/// > LOCATION;LANGUAGE=no:Tyskland
///
/// [Section 3.2.10](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.10)
#[derive(Debug)]
pub struct Language(langtag::LangTagBuf);

impl TryFrom<&[u8]> for Language {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(
            langtag::LangTagBuf::from_bytes(b.to_vec())
                .map_err(|_| ParamError::Language)?,
        ))
    }
}

impl Language {
    /// Builds a `LANGUAGE` parameter from `tag`, validating it as an
    /// [RFC 5646](https://datatracker.ietf.org/doc/html/rfc5646) language
    /// tag the same way `TryFrom<&[u8]>` does.
    pub fn new(tag: &str) -> Result<Self, ParamError> {
        Ok(Self(
            langtag::LangTagBuf::from_bytes(tag.as_bytes().to_vec())
                .map_err(|_| ParamError::Language)?,
        ))
    }
}

/// This parameter can be specified on properties with a
/// CAL-ADDRESS value type.  The parameter identifies the groups or
/// list membership for the calendar user specified by the property.
/// The parameter value is either a single calendar address in a
/// quoted-string or a COMMA-separated list of calendar addresses,
/// each in a quoted-string.  The individual calendar address
/// parameter values MUST each be specified in a quoted-string.
///
/// [Section 3.2.11](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.11)
#[derive(Debug)]
pub struct Member(Vec<CalendarUserAddress>);

impl TryFrom<&[u8]> for Member {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let mut vec = Vec::new();
        for el in b.split(|b| *b == b',') {
            vec.push(quoted(el)?.try_into()?);
        }
        Ok(Self(vec))
    }
}

impl Member {
    /// Builds a `MEMBER` parameter directly from already-typed calendar
    /// user addresses, skipping the quoted-string-list text round-trip
    /// `TryFrom<&[u8]>` requires.
    pub fn new(addresses: Vec<CalendarUserAddress>) -> Self {
        Self(addresses)
    }
}

/// This parameter can be specified on properties with a
/// CAL-ADDRESS value type.  The parameter identifies the type of
/// calendar user specified by the property.  If not specified on a
/// property that allows this parameter, the default is INDIVIDUAL.
/// Applications MUST treat x-name and iana-token values they don't
/// recognize the same way as they would the UNKNOWN value.
///
/// [Section 3.2.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.3)
#[derive(Debug, Default)]
pub enum CalendarUserType {
    /// A single person. Default.
    #[default]
    Individual,
    /// A group of calendar users.
    Group,
    /// A physical resource (e.g. a projector).
    Resource,
    /// A room resource.
    Room,
    /// The type is unknown.
    Unknown,
    /// A non-standard, `X-`-prefixed value.
    X(Text),
    /// A value registered with IANA that isn't one of the values above.
    Iana(Text),
}

/// This parameter can be specified on properties with a
/// CAL-ADDRESS value type.  The parameter identifies the
/// participation status for the calendar user specified by the
/// property value.  The parameter values differ depending on whether
/// they are associated with a group-scheduled "VEVENT", "VTODO", or
/// "VJOURNAL".  The values MUST match one of the values allowed for
/// the given calendar component.  If not specified on a property that
/// allows this parameter, the default value is NEEDS-ACTION.
/// Applications MUST treat x-name and iana-token values they don't
/// recognize the same way as they would the NEEDS-ACTION value.
///
/// Example:
///
/// > ATTENDEE;PARTSTAT=DECLINED:mailto:jsmith@example.com
///
/// [Section 3.2.12](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.12)
#[derive(Debug)]
pub enum ParticipationStatus {
    /// Status for a `VEVENT` attendee.
    Event(PartStatEvent),
    /// Status for a `VTODO` attendee.
    Todo(PartStatTodo),
    /// Status for a `VJOURNAL` attendee.
    Journal(PartStatJournal),
    /// A non-standard, `X-`-prefixed value.
    X(Text),
    /// A value registered with IANA that isn't one of the values above.
    Iana(Text),
}

impl TryFrom<&[u8]> for ParticipationStatus {
    type Error = ParamError;

    // COMPLETED and IN-PROCESS are VTODO-only; everything else is routed
    // through PartStatEvent, which covers the common superset
    // (NEEDS-ACTION, ACCEPTED, DECLINED, TENTATIVE, DELEGATED) shared
    // across VEVENT and VJOURNAL.
    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        match b {
            b"COMPLETED" => Ok(Self::Todo(PartStatTodo::Completed)),
            b"IN-PROCESS" => Ok(Self::Todo(PartStatTodo::InProcess)),
            x if x.to_ascii_uppercase().starts_with(b"X-") => {
                Ok(Self::X(x.try_into()?))
            }
            x => match PartStatEvent::try_from(x) {
                Ok(s) => Ok(Self::Event(s)),
                Err(_) => Ok(Self::Iana(x.try_into()?)),
            },
        }
    }
}

/// This parameter can be specified on properties with a
/// CAL-ADDRESS value type.  The parameter identifies the expectation
/// of a reply from the calendar user specified by the property value.
/// This parameter is used by the "Organizer" to request a
/// participation status reply from an "Attendee" of a group-scheduled
/// event or to-do.  If not specified on a property that allows this
/// parameter, the default value is FALSE.
///
/// Example:
///
/// > ATTENDEE;RSVP=TRUE:mailto:jsmith@example.com
///
/// [Section 3.2.17](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.17)
#[derive(Debug)]
pub struct Rsvp(Boolean);

/// This parameter can be specified on properties with a
/// CAL-ADDRESS value type.  The parameter specifies the calendar user
/// that is acting on behalf of the calendar user specified by the
/// property.  The parameter value MUST be a mailto URI as defined in
/// [RFC2368].  The individual calendar address parameter values MUST
/// each be specified in a quoted-string.
///
/// [Section 3.2.18](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.18)
#[derive(Debug)]
pub struct SentBy(CalendarUserAddress);

/// This parameter MUST be specified on the "DTSTART",
/// "DTEND", "DUE", "EXDATE", and "RDATE" properties when either a
/// DATE-TIME or TIME value type is specified and when the value is
/// neither a UTC or a "floating" time.  Refer to the DATE-TIME or
/// TIME value type definition for a description of UTC and "floating
/// time" formats.  This property parameter specifies a text value
/// that uniquely identifies the "VTIMEZONE" calendar component to be
/// used when evaluating the time portion of the property.  The value
/// of the "TZID" property parameter will be equal to the value of the
/// "TZID" property for the matching time zone definition.  An
/// individual "VTIMEZONE" calendar component MUST be specified for
/// each unique "TZID" parameter value specified in the iCalendar
/// object.
///
/// The parameter MUST be specified on properties with a DATE-TIME
/// value if the DATE-TIME is not either a UTC or a "floating" time.
/// Failure to include and follow VTIMEZONE definitions in iCalendar
/// objects may lead to inconsistent understanding of the local time
/// at any given location.
///
/// The presence of the SOLIDUS character as a prefix, indicates that
/// this "TZID" represents a unique ID in a globally defined time zone
/// registry (when such registry is defined).
///
/// > Note: This document does not define a naming convention for
/// > time zone identifiers.  Implementers may want to use the naming
/// > conventions defined in existing time zone specifications such
/// > as the public-domain TZ database [TZDB].  The specification of
/// > globally unique time zone identifiers is not addressed by this
/// > document and is left for future study.
///
/// The following are examples of this property parameter:
///
/// > DTSTART;TZID=America/New_York:19980119T020000
/// >
/// > DTEND;TZID=America/New_York:19980119T030000
///
/// The "TZID" property parameter MUST NOT be applied to DATE
/// properties and DATE-TIME or TIME properties whose time values are
/// specified in UTC.
///
/// The use of local time in a DATE-TIME or TIME value without the
/// "TZID" property parameter is to be interpreted as floating time,
/// regardless of the existence of "VTIMEZONE" calendar components in
/// the iCalendar object.
///
/// For more information, see the sections on the value types [DateType] and
/// [Time].
///
/// This crate stores `TZID` as opaque text rather than requiring it to name
/// a [`chrono_tz::Tz`] zone — the RFC leaves the naming convention for
/// `TZID` values unspecified (see the note above), and real-world producers
/// routinely emit values `chrono_tz` doesn't recognize: a non-IANA alias
/// (`US-Eastern`), a Windows/Exchange display name (`Eastern Standard
/// Time`), a raw UTC offset (`UTC+11`), a custom or `/`-prefixed
/// globally-unique identifier that only resolves against a local
/// `VTIMEZONE`, or a display name real-world producers wrap in DQUOTEs
/// despite `TZID`'s grammar not permitting a `quoted-string` (a wrapping
/// pair is tolerated and stripped the same way other lenient params handle
/// non-conformant quoting elsewhere in this crate). Rejecting all of that at
/// parse time (this crate's previous behavior, see issue #27) makes the
/// `TZID` *parameter* unusable for any producer that doesn't happen to name
/// a real IANA zone, even though the parameter itself is just a text
/// reference.
///
/// [`resolve`](Self::resolve) is the one place `chrono_tz` still comes in:
/// a *best-effort*, convenience lookup used only when actually computing a
/// zoned `DATE-TIME`'s real UTC instant (RFC 5545 §3.3.5). When it returns
/// `None` (a non-IANA name, a custom identifier, a globally-unique ID that
/// would need a local `VTIMEZONE` this crate doesn't evaluate offset rules
/// from), the local time is kept as `Floating` rather than guessed at —
/// no precision is fabricated, but nothing is rejected either.
///
/// [Section 3.2.19](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.19)
#[derive(Debug)]
pub struct TimeZoneIdentifier(Text);

/// This parameter can be specified on properties with a
/// CAL-ADDRESS value type.  The parameter specifies the participation
/// role for the calendar user specified by the property in the group
/// schedule calendar component.  If not specified on a property that
/// allows this parameter, the default value is REQ-PARTICIPANT.
/// Applications MUST treat x-name and iana-token values they don't
/// recognize the same way as they would the REQ-PARTICIPANT value.
///
/// Example:
///
/// > ATTENDEE;ROLE=CHAIR:mailto:mrbig@example.com
///
/// [RFC](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.16)
#[derive(Debug, Default)]
pub enum ParticipationRole {
    /// Organizer/chair of the meeting.
    Chair,
    /// Required participant. Default.
    #[default]
    ReqParticipant,
    /// Optional participant.
    OptParticipant,
    /// Receives a copy but is not expected to participate.
    NonParticipant,
    /// A non-standard, `X-`-prefixed value.
    X(Text),
    /// A value registered with IANA that isn't one of the values above.
    Iana(Text),
}

/// Participation statuses for a "VEVENT"
#[derive(Debug, Default)]
pub enum PartStatEvent {
    /// No reply has been received. Default.
    #[default]
    NeedsAction,
    /// Invitation has been accepted.
    Accepted,
    /// Invitation has been declined.
    Declined,
    /// Participation is tentative.
    Tentative,
    /// Participation has been delegated to another attendee.
    Delegated,
    /// A non-standard, `X-`-prefixed value.
    X(Text),
    /// A value registered with IANA that isn't one of the values above.
    Iana(Text),
}

/// Participation statuses for a "VTODO"
#[derive(Debug, Default)]
pub enum PartStatTodo {
    /// No reply has been received. Default.
    #[default]
    NeedsAction,
    /// To-do has been accepted.
    Accepted,
    /// To-do has been declined.
    Declined,
    /// Participation is tentative.
    Tentative,
    /// To-do has been delegated.
    Delegated,
    /// To-do has been completed.
    Completed,
    /// To-do is being worked on.
    InProcess,
    /// A non-standard, `X-`-prefixed value.
    X(Text),
    /// A value registered with IANA that isn't one of the values above.
    Iana(Text),
}

/// Participation statuses for a "VJOURNAL"
#[derive(Debug, Default)]
pub enum PartStatJournal {
    /// No reply has been received. Default.
    #[default]
    NeedsAction,
    /// Journal entry has been accepted.
    Accepted,
    /// Journal entry has been declined.
    Declined,
    /// A non-standard, `X-`-prefixed value.
    X(Text),
    /// A value registered with IANA that isn't one of the values above.
    Iana(Text),
}

/// This parameter can be specified on a property that
/// references another related calendar.  The parameter specifies the
/// hierarchical relationship type of the calendar component
/// referenced by the property.  The parameter value can be PARENT, to
/// indicate that the referenced calendar component is a superior of
/// calendar component; CHILD to indicate that the referenced calendar
/// component is a subordinate of the calendar component; or SIBLING
/// to indicate that the referenced calendar component is a peer of
/// the calendar component.  If this parameter is not specified on an
/// allowable property, the default relationship type is PARENT.
/// Applications MUST treat x-name and iana-token values they don't
/// recognize the same way as they would the PARENT value.
///
/// Example:
///
/// > RELATED-TO;RELTYPE=SIBLING:19960401-080045-4000F192713@example.com
///
/// [Section 3.2.15](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.15)
#[derive(Debug, Default)]
pub enum RelationshipType {
    /// The referenced component is a parent (superior). Default.
    #[default]
    Parent,
    /// The referenced component is a child (subordinate).
    Child,
    /// The referenced component is a sibling (peer).
    Sibling,
    /// A non-standard, `X-`-prefixed value.
    X(Text),
    /// A value registered with IANA that isn't one of the values above.
    Iana(Text),
}

/// This parameter can be specified on properties that
/// specify an alarm trigger with a "DURATION" value type.  The
/// parameter specifies whether the alarm will trigger relative to the
/// start or end of the calendar component.  The parameter value START
/// will set the alarm to trigger off the start of the calendar
/// component; the parameter value END will set the alarm to trigger
/// off the end of the calendar component.  If the parameter is not
/// specified on an allowable property, then the default is START.
///
/// Example:
///
/// > TRIGGER;RELATED=END:PT5M
///
/// [Section 3.2.14](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.14)
#[derive(Debug, Default)]
pub enum AlarmTriggerRelationship {
    /// Trigger relative to the start of the component. Default.
    #[default]
    Start,
    /// Trigger relative to the end of the component.
    End,
}

/// This parameter can be specified on a property that
/// specifies a recurrence identifier.  The parameter specifies the
/// effective range of recurrence instances that is specified by the
/// property.  The effective range is from the recurrence identifier
/// specified by the property.  If this parameter is not specified on
/// an allowed property, then the default range is the single instance
/// specified by the recurrence identifier value of the property.  The
/// parameter value can only be "THISANDFUTURE" to indicate a range
/// defined by the recurrence identifier and all subsequent instances.
/// The value "THISANDPRIOR" is deprecated by this revision of
/// iCalendar and MUST NOT be generated by applications.
///
/// Example:
///
/// > RECURRENCE-ID;RANGE=THISANDFUTURE:19980401T133000Z
///
/// [Section 3.2.13](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.13)
#[derive(Debug)]
pub enum RecurrenceIdentifierRange {
    /// Range covers the identified instance and all subsequent instances.
    ThisAndFuture,
}

/// Shorthand alias for [`RecurrenceIdentifierRange`], used by
/// [`crate::RecurrenceSet`].
pub type Range = RecurrenceIdentifierRange;

impl TryFrom<&[u8]> for CalendarUserType {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let r = match b {
            b"INDIVIDUAL" => Self::Individual,
            b"GROUP" => Self::Group,
            b"RESOURCE" => Self::Resource,
            b"ROOM" => Self::Room,
            b"UNKNOWN" => Self::Unknown,
            x => {
                if x.to_ascii_uppercase().starts_with(b"X-") {
                    Self::X(x.try_into()?)
                } else {
                    Self::Iana(x.try_into()?)
                }
            }
        };
        Ok(r)
    }
}

impl TryFrom<&[u8]> for ParticipationRole {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let r = match b {
            b"CHAIR" => Self::Chair,
            b"REQ-PARTICIPANT" => Self::ReqParticipant,
            b"OPT-PARTICIPANT" => Self::OptParticipant,
            b"NON-PARTICIPANT" => Self::NonParticipant,
            x => {
                if x.to_ascii_uppercase().starts_with(b"X-") {
                    Self::X(x.try_into()?)
                } else {
                    Self::Iana(x.try_into()?)
                }
            }
        };
        Ok(r)
    }
}

impl TryFrom<&[u8]> for PartStatEvent {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let r = match b {
            b"NEEDS-ACTION" => Self::NeedsAction,
            b"ACCEPTED" => Self::Accepted,
            b"DECLINED" => Self::Declined,
            b"TENTATIVE" => Self::Tentative,
            b"DELEGATED" => Self::Delegated,
            x => {
                if x.to_ascii_uppercase().starts_with(b"X-") {
                    Self::X(x.try_into()?)
                } else {
                    Self::Iana(x.try_into()?)
                }
            }
        };
        Ok(r)
    }
}

impl TryFrom<&[u8]> for PartStatTodo {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let r = match b {
            b"NEEDS-ACTION" => Self::NeedsAction,
            b"ACCEPTED" => Self::Accepted,
            b"DECLINED" => Self::Declined,
            b"TENTATIVE" => Self::Tentative,
            b"DELEGATED" => Self::Delegated,
            b"COMPLETED" => Self::Completed,
            b"IN-PROCESS" => Self::InProcess,
            x => {
                if x.to_ascii_uppercase().starts_with(b"X-") {
                    Self::X(x.try_into()?)
                } else {
                    Self::Iana(x.try_into()?)
                }
            }
        };
        Ok(r)
    }
}

impl TryFrom<&[u8]> for PartStatJournal {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let r = match b {
            b"NEEDS-ACTION" => Self::NeedsAction,
            b"ACCEPTED" => Self::Accepted,
            b"DECLINED" => Self::Declined,
            x => {
                if x.to_ascii_uppercase().starts_with(b"X-") {
                    Self::X(x.try_into()?)
                } else {
                    Self::Iana(x.try_into()?)
                }
            }
        };
        Ok(r)
    }
}

impl TryFrom<&[u8]> for Rsvp {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(b.try_into()?))
    }
}

impl Rsvp {
    /// Builds an `RSVP` parameter directly from a native `bool`, skipping
    /// the `"TRUE"`/`"FALSE"` text round-trip `TryFrom<&[u8]>` requires.
    pub fn new(value: bool) -> Self {
        Self(Boolean::new(value))
    }
}

impl TryFrom<&[u8]> for SentBy {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(quoted(b)?.try_into()?))
    }
}

impl SentBy {
    /// Builds a `SENT-BY` parameter directly from an already-typed
    /// [`CalendarUserAddress`], skipping the quoted-string text
    /// round-trip `TryFrom<&[u8]>` requires.
    pub fn new(address: CalendarUserAddress) -> Self {
        Self(address)
    }
}

impl TryFrom<&[u8]> for TimeZoneIdentifier {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(maybe_quoted(b).try_into()?))
    }
}

impl TimeZoneIdentifier {
    /// Builds a `TZID` parameter directly from an already-valid
    /// [`chrono_tz::Tz`], skipping the text round-trip `TryFrom<&[u8]>`
    /// requires. Infallible — every `Tz` variant has a valid IANA zone name
    /// by construction.
    pub fn new(tz: Tz) -> Self {
        Self(tz.name().into())
    }

    /// The raw `TZID` text — also used by the calendar-wide check that this
    /// parameter's value matches a `VTIMEZONE` component's own `TZID`
    /// property elsewhere in the object (RFC 5545 §3.2.19), and to compare
    /// two `TZID` parameters for equality (e.g. `EXDATE`/`RDATE` against
    /// their component's `DTSTART`) without requiring either to resolve.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Best-effort resolution against [`chrono_tz::Tz`]'s IANA database —
    /// `None` for any `TZID` text that isn't a name `chrono_tz` recognizes
    /// (see the type's doc comment). Used only to compute a zoned
    /// `DATE-TIME`'s real UTC instant (RFC 5545 §3.3.5); the `TZID` text
    /// itself is always preserved regardless of whether this resolves.
    pub(crate) fn resolve(&self) -> Option<Tz> {
        self.0.parse().ok()
    }
}

impl TryFrom<&[u8]> for RelationshipType {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let r = match b {
            b"PARENT" => Self::Parent,
            b"CHILD" => Self::Child,
            b"SIBLING" => Self::Sibling,
            x => {
                if x.to_ascii_uppercase().starts_with(b"X-") {
                    Self::X(x.try_into()?)
                } else {
                    Self::Iana(x.try_into()?)
                }
            }
        };
        Ok(r)
    }
}

impl TryFrom<&[u8]> for AlarmTriggerRelationship {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        match b {
            b"START" => Ok(Self::Start),
            b"END" => Ok(Self::End),
            _ => Err(ParamError::Malformed {
                expected: "START or END".into(),
                received: std::str::from_utf8(b).ok().map(|s| s.into()),
            }),
        }
    }
}

impl TryFrom<&[u8]> for RecurrenceIdentifierRange {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        match b {
            b"THISANDFUTURE" => Ok(Self::ThisAndFuture),
            _ => Err(ParamError::Malformed {
                expected: "THISANDFUTURE".into(),
                received: std::str::from_utf8(b).ok().map(|s| s.into()),
            }),
        }
    }
}

/// Renders as an RFC 5545 `param-value` (`paramtext / quoted-string`,
/// §3.2's grammar for a parameter whose quoting is optional): wrapped in
/// DQUOTEs when it contains a COLON, SEMICOLON, or COMMA — the characters
/// `paramtext` excludes — left bare otherwise.
trait FmtParamValue {
    fn fmt_param_value(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result;
}

impl FmtParamValue for str {
    fn fmt_param_value(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        if self.contains([':', ';', ',']) {
            write!(f, "\"{self}\"")
        } else {
            f.write_str(self)
        }
    }
}

/// Renders as a COMMA-separated list of calendar addresses, each wrapped in
/// a quoted-string as RFC 5545 requires for `DELEGATED-FROM`/`DELEGATED-TO`/
/// `MEMBER` (§3.2.4, §3.2.5, §3.2.11).
trait FmtQuotedAddressList {
    fn fmt_quoted_address_list(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result;
}

impl FmtQuotedAddressList for [CalendarUserAddress] {
    fn fmt_quoted_address_list(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for (i, addr) in self.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "\"{addr}\"")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ValueDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binary => f.write_str("BINARY"),
            Self::Uri => f.write_str("URI"),
            Self::Text => f.write_str("TEXT"),
            Self::Boolean => f.write_str("BOOLEAN"),
            Self::CalAddress => f.write_str("CAL-ADDRESS"),
            Self::Date => f.write_str("DATE"),
            Self::DateTime => f.write_str("DATE-TIME"),
            Self::Duration => f.write_str("DURATION"),
            Self::Float => f.write_str("FLOAT"),
            Self::Integer => f.write_str("INTEGER"),
            Self::Period => f.write_str("PERIOD"),
            Self::Recur => f.write_str("RECUR"),
            Self::Time => f.write_str("TIME"),
            Self::UtcOffset => f.write_str("UTC-OFFSET"),
            Self::XName(t) | Self::Iana(t) => f.write_str(t.as_str()),
        }
    }
}

impl std::fmt::Display for Altrep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", *self.0)
    }
}

impl std::fmt::Display for CommonName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_str().fmt_param_value(f)
    }
}

impl std::fmt::Display for Delegators {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_quoted_address_list(f)
    }
}

impl std::fmt::Display for Delegatees {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_quoted_address_list(f)
    }
}

impl std::fmt::Display for DirectoryEntryReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", *self.0)
    }
}

impl std::fmt::Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Bit8 => "8BIT",
            Self::Base64 => "BASE64",
        })
    }
}

impl std::fmt::Display for Fmttype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Fbtype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Free => f.write_str("FREE"),
            Self::Busy => f.write_str("BUSY"),
            Self::BusyUnavailable => f.write_str("BUSY-UNAVAILABLE"),
            Self::BusyTentative => f.write_str("BUSY-TENTATIVE"),
            Self::X(t) | Self::Iana(t) => f.write_str(t.as_str()),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Member {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_quoted_address_list(f)
    }
}

impl std::fmt::Display for CalendarUserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Individual => f.write_str("INDIVIDUAL"),
            Self::Group => f.write_str("GROUP"),
            Self::Resource => f.write_str("RESOURCE"),
            Self::Room => f.write_str("ROOM"),
            Self::Unknown => f.write_str("UNKNOWN"),
            Self::X(t) | Self::Iana(t) => f.write_str(t.as_str()),
        }
    }
}

impl std::fmt::Display for ParticipationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Event(s) => write!(f, "{s}"),
            Self::Todo(s) => write!(f, "{s}"),
            Self::Journal(s) => write!(f, "{s}"),
            Self::X(t) | Self::Iana(t) => f.write_str(t.as_str()),
        }
    }
}

impl std::fmt::Display for Rsvp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if *self.0 { "TRUE" } else { "FALSE" })
    }
}

impl std::fmt::Display for SentBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

impl std::fmt::Display for TimeZoneIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for ParticipationRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chair => f.write_str("CHAIR"),
            Self::ReqParticipant => f.write_str("REQ-PARTICIPANT"),
            Self::OptParticipant => f.write_str("OPT-PARTICIPANT"),
            Self::NonParticipant => f.write_str("NON-PARTICIPANT"),
            Self::X(t) | Self::Iana(t) => f.write_str(t.as_str()),
        }
    }
}

impl std::fmt::Display for PartStatEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NeedsAction => f.write_str("NEEDS-ACTION"),
            Self::Accepted => f.write_str("ACCEPTED"),
            Self::Declined => f.write_str("DECLINED"),
            Self::Tentative => f.write_str("TENTATIVE"),
            Self::Delegated => f.write_str("DELEGATED"),
            Self::X(t) | Self::Iana(t) => f.write_str(t.as_str()),
        }
    }
}

impl std::fmt::Display for PartStatTodo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NeedsAction => f.write_str("NEEDS-ACTION"),
            Self::Accepted => f.write_str("ACCEPTED"),
            Self::Declined => f.write_str("DECLINED"),
            Self::Tentative => f.write_str("TENTATIVE"),
            Self::Delegated => f.write_str("DELEGATED"),
            Self::Completed => f.write_str("COMPLETED"),
            Self::InProcess => f.write_str("IN-PROCESS"),
            Self::X(t) | Self::Iana(t) => f.write_str(t.as_str()),
        }
    }
}

impl std::fmt::Display for PartStatJournal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NeedsAction => f.write_str("NEEDS-ACTION"),
            Self::Accepted => f.write_str("ACCEPTED"),
            Self::Declined => f.write_str("DECLINED"),
            Self::X(t) | Self::Iana(t) => f.write_str(t.as_str()),
        }
    }
}

impl std::fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parent => f.write_str("PARENT"),
            Self::Child => f.write_str("CHILD"),
            Self::Sibling => f.write_str("SIBLING"),
            Self::X(t) | Self::Iana(t) => f.write_str(t.as_str()),
        }
    }
}

impl std::fmt::Display for AlarmTriggerRelationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Start => "START",
            Self::End => "END",
        })
    }
}

impl std::fmt::Display for RecurrenceIdentifierRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ThisAndFuture => "THISANDFUTURE",
        })
    }
}

/// This parameter is used to specify different ways in which an image for
/// a calendar or component can be displayed.
///
/// Applications MUST handle a value they don't recognize (an `x-name` or
/// `iana-token` other than the four below) the same way they'd handle the
/// default, `BADGE`.
///
/// Example:
///
/// > IMAGE;DISPLAY=BADGE,THUMBNAIL;VALUE=URI:https://example.com/image.png
///
/// [Section 6.1](https://datatracker.ietf.org/doc/html/rfc7986#section-6.1)
#[derive(Debug)]
pub struct ImageDisplay(Vec<ImageDisplayValue>);

impl ImageDisplay {
    /// Constructs a `DISPLAY` parameter from its list of values.
    pub fn new(values: Vec<ImageDisplayValue>) -> Self {
        Self(values)
    }
}

/// One value of an [`ImageDisplay`] list.
#[derive(Debug)]
pub enum ImageDisplayValue {
    /// A smaller image inline with the text (the default if `DISPLAY` is
    /// absent).
    Badge,
    /// A full image replacement for the text.
    Graphic,
    /// A full-sized image.
    Fullsize,
    /// A smaller image thumbnail.
    Thumbnail,
    /// A non-standard, `X-`-prefixed value.
    X(Text),
    /// A value registered with IANA that isn't one of the values above.
    Iana(Text),
}

impl TryFrom<&[u8]> for ImageDisplay {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let mut vec = Vec::new();
        for el in b.split(|b| *b == b',') {
            vec.push(el.try_into()?);
        }
        Ok(Self(vec))
    }
}

impl TryFrom<&[u8]> for ImageDisplayValue {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let r = match b {
            b"BADGE" => Self::Badge,
            b"GRAPHIC" => Self::Graphic,
            b"FULLSIZE" => Self::Fullsize,
            b"THUMBNAIL" => Self::Thumbnail,
            x => {
                if x.to_ascii_uppercase().starts_with(b"X-") {
                    Self::X(x.try_into()?)
                } else {
                    Self::Iana(x.try_into()?)
                }
            }
        };
        Ok(r)
    }
}

impl std::fmt::Display for ImageDisplayValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Badge => f.write_str("BADGE"),
            Self::Graphic => f.write_str("GRAPHIC"),
            Self::Fullsize => f.write_str("FULLSIZE"),
            Self::Thumbnail => f.write_str("THUMBNAIL"),
            Self::X(t) | Self::Iana(t) => f.write_str(t.as_str()),
        }
    }
}

impl std::fmt::Display for ImageDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, v) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{v}")?;
        }
        Ok(())
    }
}

/// This parameter identifies the type of conferencing system access that a
/// `CONFERENCE` property's URI provides.
///
/// Example:
///
/// > CONFERENCE;FEATURE=PHONE,MODERATOR;VALUE=URI:tel:+1-412-555-0123,,,654321
///
/// [Section 6.3](https://datatracker.ietf.org/doc/html/rfc7986#section-6.3)
#[derive(Debug)]
pub struct Feature(Vec<FeatureValue>);

impl Feature {
    /// Constructs a `FEATURE` parameter from its list of values.
    pub fn new(values: Vec<FeatureValue>) -> Self {
        Self(values)
    }
}

/// One value of a [`Feature`] list.
#[derive(Debug)]
pub enum FeatureValue {
    /// An audio conference.
    Audio,
    /// A live chat conference.
    Chat,
    /// A blog or Atom feed.
    Feed,
    /// The moderator's dial-in or access code.
    Moderator,
    /// A phone conference.
    Phone,
    /// A screen-sharing conference.
    Screen,
    /// A video conference.
    Video,
    /// A non-standard, `X-`-prefixed value.
    X(Text),
    /// A value registered with IANA that isn't one of the values above.
    Iana(Text),
}

impl TryFrom<&[u8]> for Feature {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let mut vec = Vec::new();
        for el in b.split(|b| *b == b',') {
            vec.push(el.try_into()?);
        }
        Ok(Self(vec))
    }
}

impl TryFrom<&[u8]> for FeatureValue {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let r = match b {
            b"AUDIO" => Self::Audio,
            b"CHAT" => Self::Chat,
            b"FEED" => Self::Feed,
            b"MODERATOR" => Self::Moderator,
            b"PHONE" => Self::Phone,
            b"SCREEN" => Self::Screen,
            b"VIDEO" => Self::Video,
            x => {
                if x.to_ascii_uppercase().starts_with(b"X-") {
                    Self::X(x.try_into()?)
                } else {
                    Self::Iana(x.try_into()?)
                }
            }
        };
        Ok(r)
    }
}

impl std::fmt::Display for FeatureValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Audio => f.write_str("AUDIO"),
            Self::Chat => f.write_str("CHAT"),
            Self::Feed => f.write_str("FEED"),
            Self::Moderator => f.write_str("MODERATOR"),
            Self::Phone => f.write_str("PHONE"),
            Self::Screen => f.write_str("SCREEN"),
            Self::Video => f.write_str("VIDEO"),
            Self::X(t) | Self::Iana(t) => f.write_str(t.as_str()),
        }
    }
}

impl std::fmt::Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, v) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{v}")?;
        }
        Ok(())
    }
}

/// This parameter provides a human-readable label for a `CONFERENCE`
/// property's access URI, e.g. to distinguish a moderator dial-in from an
/// attendee one.
///
/// Example:
///
/// > CONFERENCE;LABEL=Attendee dial-in;VALUE=URI:tel:+1-412-555-0123
///
/// [Section 6.4](https://datatracker.ietf.org/doc/html/rfc7986#section-6.4)
#[derive(Debug)]
pub struct Label(Text);

impl Label {
    /// Constructs a `LABEL` parameter from its text.
    pub fn new(value: Text) -> Self {
        Self(value)
    }
}

impl TryFrom<&[u8]> for Label {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(maybe_quoted(b).try_into()?))
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}

/// A single property parameter (`ALTREP`, `LANGUAGE`, `TZID`, ...) failed to
/// parse into its typed representation. Not to be confused with
/// [`ParameterError`], which covers the surrounding `*(";" param)` list
/// syntax (missing `=`, an unmodeled `NAME`, ...).
#[derive(Debug, Error)]
pub enum ParamError {
    /// `RRULE`'s `FREQ` value didn't match one of the defined frequency names.
    #[error("invalid frequency: {0}")]
    InvalidFreq(String),
    /// A weekday value (e.g. in `BYDAY`) didn't match one of the two-letter
    /// weekday abbreviations.
    #[error("invalid weekday: {0}")]
    InvalidWeekday(String),
    /// The `LANGUAGE` parameter's value isn't a well-formed language tag.
    #[error("malformed LANGUAGE tag")]
    Language,
    /// The parameter value isn't a valid `quoted-string`.
    #[error("not a quoted-string value")]
    QuotedString,
    /// The parameter value didn't match its expected grammar.
    #[error("parameter parsing failed. Expected {expected}, got {received:?}")]
    Malformed {
        /// What the parameter value is supposed to be
        expected: String,
        /// What we actually received
        received: Option<String>,
    },
    /// A param that's a thin wrapper over a `values.rs` type (e.g.
    /// `Altrep(Uri)`, `CommonName(Text)`) failed at that inner value's own
    /// parse step.
    #[error(transparent)]
    Value(#[from] ValueError),

    /// Encoding error surfaced while decoding a param's raw bytes as UTF-8.
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}

/// Adds a `value()` read accessor and a `Deref` to the wrapped value for a
/// single-value parameter newtype, mirroring `impl_value_accessor!` on the
/// property side. The inner field stays private.
macro_rules! impl_param_value {
    ($ty:ident, $value_ty:ty) => {
        impl $ty {
            /// Returns the parameter's parsed value.
            pub fn value(&self) -> &$value_ty {
                &self.0
            }
        }

        impl std::ops::Deref for $ty {
            type Target = $value_ty;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

/// Like [`impl_param_value!`], for a parameter whose value is a
/// comma-separated list: reads as a slice.
macro_rules! impl_param_list_value {
    ($ty:ident, $item_ty:ty) => {
        impl $ty {
            /// Returns the parameter's parsed values.
            pub fn value(&self) -> &[$item_ty] {
                &self.0
            }
        }

        impl std::ops::Deref for $ty {
            type Target = [$item_ty];

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

impl_param_value!(Altrep, Uri);
impl_param_value!(CommonName, Text);
impl_param_value!(DirectoryEntryReference, Uri);
impl_param_value!(Fmttype, MediaType);
impl_param_value!(SentBy, CalendarUserAddress);
impl_param_value!(Label, Text);
impl_param_list_value!(Delegators, CalendarUserAddress);
impl_param_list_value!(Delegatees, CalendarUserAddress);
impl_param_list_value!(Member, CalendarUserAddress);
impl_param_list_value!(ImageDisplay, ImageDisplayValue);
impl_param_list_value!(Feature, FeatureValue);

impl Rsvp {
    /// Returns `true` if a reply was requested (`RSVP=TRUE`).
    pub fn value(&self) -> bool {
        *self.0
    }
}

impl Language {
    /// The language tag as text, e.g. `"en-US"`.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tzid_accepts_a_clean_iana_zone_name() {
        let tz = TimeZoneIdentifier::try_from(b"America/New_York".as_slice())
            .unwrap();
        assert_eq!(tz.resolve(), Some(Tz::America__New_York));
    }

    // Issue #27 bucket 3: TZID's grammar (RFC 5545 §3.2.19) is free text —
    // this crate no longer rejects a TZID parameter for not naming a real
    // chrono_tz zone (see the doc comment on `TimeZoneIdentifier`). Each of
    // these fixture-derived shapes must still parse (`TryFrom` succeeds),
    // but `resolve()` returns `None` since chrono_tz genuinely can't
    // compute offset rules for them.

    #[test]
    fn tzid_accepts_a_raw_utc_offset_but_does_not_resolve_it() {
        // tests/fixtures/collective-icalendar/calendars/issue_218_bad_tzid.ics
        let tz = TimeZoneIdentifier::try_from(b"UTC+11".as_slice()).unwrap();
        assert_eq!(tz.as_str(), "UTC+11");
        assert_eq!(tz.resolve(), None);
    }

    #[test]
    fn tzid_accepts_a_space_instead_of_underscore_but_does_not_resolve_it() {
        // tests/fixtures/collective-icalendar/timezones/
        // issue_55_parse_error_on_utc_offset_with_seconds.ics
        let tz =
            TimeZoneIdentifier::try_from(b"America/Los Angeles".as_slice())
                .unwrap();
        assert_eq!(tz.as_str(), "America/Los Angeles");
        assert_eq!(tz.resolve(), None);
    }

    #[test]
    fn tzid_accepts_a_non_ascii_display_name_but_does_not_resolve_it() {
        // tests/fixtures/collective-icalendar/timezones/
        // issue_237_brazilia_standard.ics
        let tz =
            TimeZoneIdentifier::try_from("(UTC-03:00) Brasília".as_bytes())
                .unwrap();
        assert_eq!(tz.as_str(), "(UTC-03:00) Brasília");
        assert_eq!(tz.resolve(), None);
    }

    #[test]
    fn tzid_accepts_a_windows_exchange_zone_name_but_does_not_resolve_it() {
        // tests/fixtures/collective-icalendar/calendars/
        // issue_836_do_not_quote_tzid.ics
        let tz =
            TimeZoneIdentifier::try_from(b"Eastern Standard Time".as_slice())
                .unwrap();
        assert_eq!(tz.as_str(), "Eastern Standard Time");
        assert_eq!(tz.resolve(), None);
    }

    /// `TZID`'s grammar (§3.2.19) doesn't permit a `quoted-string` at all,
    /// but real-world producers (Windows/Exchange in particular) wrap
    /// display-name `TZID`s in DQUOTEs anyway — tolerated the same way
    /// other lenient params handle optional quoting, and stripped rather
    /// than kept as part of the stored text.
    #[test]
    fn tzid_strips_a_non_conformant_wrapping_quote_pair() {
        // tests/fixtures/collective-icalendar/calendars/
        // issue_156_RDATE_with_PERIOD_TZID_khal.ics
        let tz = TimeZoneIdentifier::try_from(
            b"\"Central Standard Time\"".as_slice(),
        )
        .unwrap();
        assert_eq!(tz.as_str(), "Central Standard Time");
    }

    /// A `/`-prefixed globally-unique `TZID` (§3.2.19's `tzidprefix`) is
    /// preserved verbatim, prefix included — this crate has no registry to
    /// resolve it against, so it's opaque text like any other unresolvable
    /// `TZID`.
    #[test]
    fn tzid_preserves_a_globally_unique_slash_prefix() {
        // tests/fixtures/collective-icalendar/calendars/
        // issue_313_globally_unique_tzid.ics
        let tz = TimeZoneIdentifier::try_from(
            b"/freeassociation.sourceforge.net/Europe/Berlin".as_slice(),
        )
        .unwrap();
        assert_eq!(
            tz.as_str(),
            "/freeassociation.sourceforge.net/Europe/Berlin"
        );
        assert_eq!(tz.resolve(), None);
    }

    #[test]
    fn tzid_new_matches_the_parsed_equivalent() {
        let built = TimeZoneIdentifier::new(Tz::America__New_York);
        let parsed =
            TimeZoneIdentifier::try_from(b"America/New_York".as_slice())
                .unwrap();
        assert_eq!(built.to_string(), parsed.to_string());
    }

    #[test]
    fn altrep_new_matches_the_parsed_equivalent() {
        let built =
            Altrep::new(Uri::parse("cid:part1.0001@example.org").unwrap());
        let parsed =
            Altrep::try_from(b"\"cid:part1.0001@example.org\"".as_slice())
                .unwrap();
        assert_eq!(built.to_string(), parsed.to_string());
    }

    #[test]
    fn common_name_new_matches_the_parsed_equivalent() {
        let built = CommonName::new(Text::from("John Smith"));
        let parsed = CommonName::try_from(b"John Smith".as_slice()).unwrap();
        assert_eq!(built.to_string(), parsed.to_string());
    }

    #[test]
    fn delegators_new_matches_the_parsed_equivalent() {
        let addr = CalendarUserAddress::new(
            Uri::parse("mailto:jsmith@example.com").unwrap(),
        )
        .unwrap();
        let built = Delegators::new(vec![addr]);
        let parsed =
            Delegators::try_from(b"\"mailto:jsmith@example.com\"".as_slice())
                .unwrap();
        assert_eq!(built.to_string(), parsed.to_string());
    }

    #[test]
    fn delegatees_new_matches_the_parsed_equivalent() {
        let addr = CalendarUserAddress::new(
            Uri::parse("mailto:jdoe@example.com").unwrap(),
        )
        .unwrap();
        let built = Delegatees::new(vec![addr]);
        let parsed =
            Delegatees::try_from(b"\"mailto:jdoe@example.com\"".as_slice())
                .unwrap();
        assert_eq!(built.to_string(), parsed.to_string());
    }

    #[test]
    fn directory_entry_reference_new_matches_the_parsed_equivalent() {
        let built = DirectoryEntryReference::new(
            Uri::parse("ldap://example.com:6666/o=ABC%20Industries").unwrap(),
        );
        let parsed = DirectoryEntryReference::try_from(
            b"\"ldap://example.com:6666/o=ABC%20Industries\"".as_slice(),
        )
        .unwrap();
        assert_eq!(built.to_string(), parsed.to_string());
    }

    #[test]
    fn fmttype_new_matches_the_parsed_equivalent() {
        let built = Fmttype::new(MediaType::new("application", "msword"));
        let parsed =
            Fmttype::try_from(b"application/msword".as_slice()).unwrap();
        assert_eq!(built.to_string(), parsed.to_string());
    }

    #[test]
    fn language_new_matches_the_parsed_equivalent() {
        let built = Language::new("en-US").unwrap();
        let parsed = Language::try_from(b"en-US".as_slice()).unwrap();
        assert_eq!(built.to_string(), parsed.to_string());
    }

    #[test]
    fn language_new_rejects_a_malformed_tag() {
        assert!(matches!(Language::new(""), Err(ParamError::Language)));
    }

    #[test]
    fn member_new_matches_the_parsed_equivalent() {
        let addr = CalendarUserAddress::new(
            Uri::parse("mailto:ietf-calsch@example.org").unwrap(),
        )
        .unwrap();
        let built = Member::new(vec![addr]);
        let parsed =
            Member::try_from(b"\"mailto:ietf-calsch@example.org\"".as_slice())
                .unwrap();
        assert_eq!(built.to_string(), parsed.to_string());
    }

    #[test]
    fn rsvp_new_matches_the_parsed_equivalent() {
        let built = Rsvp::new(true);
        let parsed = Rsvp::try_from(b"TRUE".as_slice()).unwrap();
        assert_eq!(built.to_string(), parsed.to_string());
    }

    #[test]
    fn sent_by_new_matches_the_parsed_equivalent() {
        let addr = CalendarUserAddress::new(
            Uri::parse("mailto:jsmith@example.com").unwrap(),
        )
        .unwrap();
        let built = SentBy::new(addr);
        let parsed =
            SentBy::try_from(b"\"mailto:jsmith@example.com\"".as_slice())
                .unwrap();
        assert_eq!(built.to_string(), parsed.to_string());
    }
}
