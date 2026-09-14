pub use crate::values::Recur;
use crate::{
    ast::{split_once, strip_quoted_string},
    properties::ParameterError,
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
    X(Text),
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
    X(Text),
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
    X(Text),
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
/// [Section 3.2.19](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.19)
#[derive(Debug, Clone)]
pub struct TimeZoneIdentifier(Tz);

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
    X(Text),
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
    X(Text),
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
    X(Text),
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
    X(Text),
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
    X(Text),
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

impl TryFrom<&[u8]> for SentBy {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(quoted(b)?.try_into()?))
    }
}

impl TryFrom<&[u8]> for TimeZoneIdentifier {
    type Error = ParamError;

    fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
        let s = std::str::from_utf8(b)?;
        let tz: Tz = s.parse().map_err(|_| ParamError::Malformed {
            expected: "IANA timezone identifier".into(),
            received: Some(s.into()),
        })?;
        Ok(Self(tz))
    }
}

impl TimeZoneIdentifier {
    /// The parsed [`chrono_tz::Tz`] itself — used to resolve a zoned
    /// `DATE-TIME` value against this `TZID`'s real offset rules (RFC 5545
    /// §3.3.5), rather than the host machine's own time zone.
    pub(crate) fn tz(&self) -> Tz {
        self.0
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

/// Convenience alias for the parsed param vector
pub type Params = Vec<PropertyParams>;

/// Any property parameter defined in Section 3.2 of RFC 5545.
///
/// A property can have attributes with which it is associated.
/// These "property parameters" contain meta-information about the
/// property or the property value.  Property parameters are specified
/// on a semicolon-separated list on the line of the property.
/// Property parameter values that contain the COLON, SEMICOLON, or
/// COMMA character separators MUST be specified as quoted-string
/// values.  Property values that do not contain any character
/// delimiters do not need to be quoted but may be.  Some property
/// parameter values are defined using a list of values.  In these
/// cases, the parameter value is defined as a COMMA-separated list of
/// values.
///
/// Example:
///
/// > DTSTART;TZID=America/New_York;VALUE=DATE-TIME:19980119T020000
///
/// [Section 3.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.2)
#[derive(Debug)]
enum PropertyParams {
    /// Alternate text representation URI (`ALTREP`). §3.2.1
    Altrep(Altrep),
    /// Common name for a calendar user (`CN`). §3.2.2
    CommonName(CommonName),
    /// Calendar user type (`CUTYPE`). §3.2.3
    CalendarUserType(CalendarUserType),
    /// Delegators — who delegated to this attendee (`DELEGATED-FROM`). §3.2.4
    Delegators(Delegators),
    /// Delegatees — to whom this attendee delegated (`DELEGATED-TO`). §3.2.5
    Delegatees(Delegatees),
    /// Directory entry reference URI (`DIR`). §3.2.6
    DirectoryEntryReference(DirectoryEntryReference),
    /// Inline encoding scheme (`ENCODING`). §3.2.7
    Encoding(Encoding),
    /// MIME media type of a referenced object (`FMTTYPE`). §3.2.8
    Fmttype(Fmttype),
    /// Free/busy time type (`FBTYPE`). §3.2.9
    Fbtype(Fbtype),
    /// Language of property text (`LANGUAGE`). §3.2.10
    Language(Language),
    /// Group/list membership addresses (`MEMBER`). §3.2.11
    Member(Member),
    /// Participation status (`PARTSTAT`). §3.2.12
    ParticipationStatus(ParticipationStatus),
    /// Recurrence identifier range (`RANGE`). §3.2.13
    RecurrenceIdentifierRange(RecurrenceIdentifierRange),
    /// Alarm trigger relationship (`RELATED`). §3.2.14
    AlarmTriggerRelationship(AlarmTriggerRelationship),
    /// Hierarchical relationship type (`RELTYPE`). §3.2.15
    RelationshipType(RelationshipType),
    /// Participation role (`ROLE`). §3.2.16
    ParticipationRole(ParticipationRole),
    /// Reply requested flag (`RSVP`). §3.2.17
    Rsvp(Rsvp),
    /// Acting-on-behalf-of address (`SENT-BY`). §3.2.18
    SentBy(SentBy),
    /// Time zone identifier (`TZID`). §3.2.19
    TimeZoneIdentifier(TimeZoneIdentifier),
    /// Explicit value type (`VALUE`). §3.2.20
    ValueDataType(ValueDataType),
    /// Non-standard `X-`-prefixed parameter.
    XName {
        name: crate::values::Text,
        value: crate::values::Text,
    },
    /// IANA-registered parameter not listed above.
    Iana {
        name: crate::values::Text,
        value: crate::values::Text,
    },
}

impl PropertyParams {
    fn parse(b: &[u8]) -> Result<Vec<Self>, ParameterError> {
        let mut out = Vec::new();
        // need to split by SEMICOLON BUT a SEMICOLON that is NOT in DOUBLE
        // QUOTES
        for s in b.split(|b| *b == b';') {
            let (n, v) = split_once(s, b'=').ok_or_else(|| {
                ParameterError::Malformed {
                    expected: "NAME=VALUE".into(),
                    received: std::str::from_utf8(s).ok().map(Into::into),
                }
            })?;
            let res = match n {
                b"ALTREP" => Self::Altrep(v.try_into()?),
                b"CN" => Self::CommonName(v.try_into()?),
                b"CUTYPE" => Self::CalendarUserType(v.try_into()?),
                b"DELEGATED-FROM" => Self::Delegators(v.try_into()?),
                b"DELEGATED-TO" => Self::Delegatees(v.try_into()?),
                b"DIR" => Self::DirectoryEntryReference(v.try_into()?),
                b"ENCODING" => Self::Encoding(v.try_into()?),
                b"FMTTYPE" => Self::Fmttype(v.try_into()?),
                b"FBTYPE" => Self::Fbtype(v.try_into()?),
                b"LANGUAGE" => Self::Language(v.try_into()?),
                b"MEMBER" => Self::Member(v.try_into()?),
                b"PARTSTAT" => Self::ParticipationStatus(v.try_into()?),
                b"RANGE" => Self::RecurrenceIdentifierRange(v.try_into()?),
                b"RELATED" => Self::AlarmTriggerRelationship(v.try_into()?),
                b"RELTYPE" => Self::RelationshipType(v.try_into()?),
                b"ROLE" => Self::ParticipationRole(v.try_into()?),
                b"RSVP" => Self::Rsvp(v.try_into()?),
                b"SENT-BY" => Self::SentBy(v.try_into()?),
                b"TZID" => Self::TimeZoneIdentifier(v.try_into()?),
                b"VALUE" => Self::ValueDataType(v.try_into()?),
                x => {
                    if x.to_ascii_uppercase().starts_with(b"X-") {
                        Self::XName {
                            name: x.try_into()?,
                            value: v.try_into()?,
                        }
                    } else {
                        Self::Iana {
                            name: x.try_into()?,
                            value: v.try_into()?,
                        }
                    }
                }
            };
            out.push(res);
        }
        Ok(out)
    }
}

/// A single property parameter (`ALTREP`, `LANGUAGE`, `TZID`, ...) failed to
/// parse into its typed representation. Not to be confused with
/// [`ParameterError`], which covers the surrounding `*(";" param)` list
/// syntax (missing `=`, an unmodeled `NAME`, ...).
#[derive(Debug, Error)]
pub enum ParamError {
    #[error("invalid frequency: {0}")]
    InvalidFreq(String),
    #[error("invalid weekday: {0}")]
    InvalidWeekday(String),
    #[error("malformed LANGUAGE tag")]
    Language,
    #[error("not a quoted-string value")]
    QuotedString,
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
