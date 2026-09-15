mod lexer;
pub mod parser;
mod token;
pub(crate) use lexer::{Lexer, LexerError};
use parser::{ParseError, ParseResult};

use crate::{
    Calendar,
    calendar::Component as CalComponent,
    components::{
        alarm::Alarm,
        event::Event,
        free_busy::FreeBusy,
        journal::Journal,
        timezone::{Timezone, TzProp},
        todo::Todo,
        unknown::UnknownComponent,
    },
    properties::*,
    values::{DateOrDatetime, DateTimePeriod},
};
use std::collections::HashSet;

/// Splits a Bytes vector by given pattern. Error-agnostic — callers map the
/// `None` case to whichever error type fits their layer (`ValueError`,
/// `ParamError`, `ParameterError`, `ParseError`, ...).
pub(crate) fn split_once(b: &[u8], needle: u8) -> Option<(&[u8], &[u8])> {
    b.iter()
        .position(|b| *b == needle)
        .map(|pos| (&b[..pos], &b[pos + 1..]))
}

/// [Case-insensitively](https://datatracker.ietf.org/doc/html/rfc5545#section-3.5) matches the name to a given pattern
pub(crate) fn match_name(b: &[u8], pat: &[u8]) -> bool {
    b.to_ascii_uppercase() == pat
}

/// Finds the byte offset of the first unquoted occurrence of `needle` in
/// `b`. A `needle` byte between two DQUOTE (`"`) characters doesn't count,
/// since content lines and parameter values MAY contain the character
/// they're normally split on once quoted (e.g. a `;` inside an `ALTREP`
/// URI, or the `:` that starts the value appearing inside a quoted
/// parameter value).
pub(crate) fn find_unquoted(b: &[u8], needle: u8) -> Option<usize> {
    let mut in_quotes = false;
    for (i, &byte) in b.iter().enumerate() {
        match byte {
            b'"' => in_quotes = !in_quotes,
            b if b == needle && !in_quotes => return Some(i),
            _ => {}
        }
    }
    None
}

/// Splits `b` on every *unescaped* occurrence of `needle`, per the
/// BACKSLASH-escaping grammar of [Section
/// 3.3.11](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.11): a
/// `needle` byte immediately preceded by an (unescaped) `\` is part of an
/// escape sequence, not a delimiter, and is left untouched for the
/// caller — typically [`crate::values::Text`]'s own unescaping — to decode.
/// Used to COMMA-split `TEXT`-valued lists (e.g. `CATEGORIES`) without
/// splitting inside an escaped `\,`.
pub(crate) fn split_unescaped(bytes: &[u8], needle: u8) -> Vec<&[u8]> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut escaped = false;
    for (i, &byte) in bytes.iter().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' => escaped = true,
            b if b == needle => {
                parts.push(&bytes[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&bytes[start..]);
    parts
}

/// Checks if a given value is in quotes and returns that value with the
/// quotes stripped. `None` if it isn't quoted — error-agnostic, see
/// [`split_once`].
pub(crate) fn strip_quoted_string(v: &[u8]) -> Option<&[u8]> {
    let needle = &[b'"'];

    v.strip_prefix(needle).and_then(|s| s.strip_suffix(needle))
}

/// Decodes the CARET-escape sequences [RFC 6868](https://datatracker.ietf.org/doc/html/rfc6868)
/// defines for *parameter* values (distinct from the BACKSLASH-escaping
/// [Section 3.3.11](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.11)
/// defines for `TEXT` property values, see [`crate::values::Text`]'s own
/// unescaping): `^n` decodes to a literal newline, `^'` to a literal `"`,
/// and `^^` to a literal `^`. A `^` followed by anything else is not a
/// defined sequence, so per RFC 6868 Section 3.2 ("the character sequence
/// should be left as it is") both bytes are left untouched. Operates on the
/// already-`;`/`=`-split parameter value (i.e. after [`crate::properties::param_value`]
/// has isolated it), not on the raw content line — a `^` has no special
/// meaning outside a parameter value.
pub(crate) fn decode_caret(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut iter = bytes.iter().copied().peekable();
    while let Some(b) = iter.next() {
        if b != b'^' {
            out.push(b);
            continue;
        }
        match iter.peek() {
            Some(b'n') => {
                out.push(b'\n');
                iter.next();
            }
            Some(b'\'') => {
                out.push(b'"');
                iter.next();
            }
            Some(b'^') => {
                out.push(b'^');
                iter.next();
            }
            _ => out.push(b'^'),
        }
    }
    out
}

#[cfg(test)]
mod decode_caret_tests {
    use super::*;

    #[test]
    fn decodes_newline_quote_and_caret() {
        assert_eq!(decode_caret(b"^n"), b"\n");
        assert_eq!(decode_caret(b"^'"), b"\"");
        assert_eq!(decode_caret(b"^^"), b"^");
    }

    #[test]
    fn decodes_the_fixture_example() {
        // collective-icalendar/calendars/rfc_6868.ics: ALL=^^^'^n
        assert_eq!(decode_caret(b"^^^'^n"), b"^\"\n");
    }

    #[test]
    fn decodes_caret_quote_inside_running_text() {
        // ATTENDEE;CN=George Herman ^'Babe^' Ruth
        assert_eq!(
            decode_caret(b"George Herman ^'Babe^' Ruth"),
            b"George Herman \"Babe\" Ruth"
        );
    }

    #[test]
    fn undefined_sequence_is_left_untouched() {
        // RFC 6868 3.2: "^" followed by anything else is left as-is.
        assert_eq!(decode_caret(b"^a"), b"^a");
    }

    #[test]
    fn trailing_lone_caret_is_left_untouched() {
        assert_eq!(decode_caret(b"^"), b"^");
    }

    #[test]
    fn text_without_carets_is_unchanged() {
        assert_eq!(decode_caret(b"plain value"), b"plain value");
    }
}

/// The calendar component carried by an [`crate::ICalendar`] object.
///
/// Each variant corresponds to a component type defined in RFC 5545 Section
/// 3.6.
#[derive(Debug)]
enum Component {
    /// A scheduled event (`VEVENT`).
    Event(EventBuilder),
    /// A to-do task (`VTODO`).
    Todo(TodoBuilder),
    /// A journal entry (`VJOURNAL`).
    Journal(JournalBuilder),
    /// Free/busy time information (`VFREEBUSY`).
    FreeBusy(FreeBusyBuilder),
    /// Time zone definition (`VTIMEZONE`).
    Timezone(TimezoneBuilder),
    /// An unrecognized `iana-comp`/`x-comp` (RFC 5545 §3.6). See
    /// [`UnknownComponentBuilder`].
    Unknown(UnknownComponentBuilder),
}

/// Builder for [`UnknownComponent`] — an unrecognized top-level
/// `iana-comp`/`x-comp` (RFC 5545 §3.6). Unlike every other builder in
/// this module, it has no typed alphabet of legal properties to ingest
/// into: [`crate::ast::parser::Parser::component`] short-circuits before
/// the shared property-ingest loop for this case (see that function),
/// filling this builder in one shot instead via its own dedicated parse
/// routine. `build` is therefore infallible — there's nothing left to
/// validate.
#[derive(Debug)]
struct UnknownComponentBuilder {
    name: Vec<u8>,
    lines: Vec<Vec<u8>>,
    components: Vec<UnknownComponentBuilder>,
}

impl UnknownComponentBuilder {
    fn new(name: Vec<u8>) -> Self {
        Self {
            name,
            lines: Vec::new(),
            components: Vec::new(),
        }
    }

    fn build(self) -> UnknownComponent {
        UnknownComponent {
            name: String::from_utf8_lossy(&self.name).into_owned(),
            lines: self
                .lines
                .into_iter()
                .map(|l| String::from_utf8_lossy(&l).into_owned())
                .collect(),
            components: self
                .components
                .into_iter()
                .map(UnknownComponentBuilder::build)
                .collect(),
        }
    }
}

impl From<UnknownComponentBuilder> for Component {
    fn from(value: UnknownComponentBuilder) -> Self {
        Self::Unknown(value)
    }
}

impl From<EventBuilder> for Component {
    fn from(value: EventBuilder) -> Self {
        Self::Event(value)
    }
}
impl From<TodoBuilder> for Component {
    fn from(value: TodoBuilder) -> Self {
        Self::Todo(value)
    }
}

impl From<JournalBuilder> for Component {
    fn from(value: JournalBuilder) -> Self {
        Self::Journal(value)
    }
}

impl From<FreeBusyBuilder> for Component {
    fn from(value: FreeBusyBuilder) -> Self {
        Self::FreeBusy(value)
    }
}

impl From<TimezoneBuilder> for Component {
    fn from(value: TimezoneBuilder) -> Self {
        Self::Timezone(value)
    }
}

impl Component {
    /// Routes one already-parsed [`Property`] into the matching builder's
    /// own fields. Never actually called on `Self::Unknown` — see
    /// [`UnknownComponentBuilder`]'s docs — but a `ParseResult` still
    /// needs producing to keep this exhaustive.
    fn ingest(&mut self, p: Property) -> ParseResult<()> {
        match self {
            Self::Event(b) => b.ingest(p),
            Self::Todo(b) => b.ingest(p),
            Self::Journal(b) => b.ingest(p),
            Self::FreeBusy(b) => b.ingest(p),
            Self::Timezone(b) => b.ingest(p),
            Self::Unknown(_) => Err(ParseError::UnknownComponent),
        }
    }

    /// Routes a fully-parsed `VALARM` sub-component (see
    /// [`crate::ast::parser::Parser::alarm`]) into a builder that's allowed
    /// to contain one — `VEVENT`/`VTODO` only (RFC 5545 §3.6.1, §3.6.2).
    /// No other component can legally contain a `VALARM`, so this is a
    /// plain match rather than a trait every builder has to implement.
    fn ingest_alarm(&mut self, alarm: AlarmBuilder) -> ParseResult<()> {
        match self {
            Self::Event(b) => Ok(b.alarms.push(alarm)),
            Self::Todo(b) => Ok(b.alarms.push(alarm)),
            _ => Err(ParseError::UnexpectedComponent("VALARM")),
        }
    }

    /// Routes a fully-parsed `STANDARD`/`DAYLIGHT` sub-component (see
    /// [`crate::ast::parser::Parser::tz_observance`]) into the matching
    /// observance list — `VTIMEZONE` only (RFC 5545 §3.6.5).
    fn ingest_tz_observance(
        &mut self,
        kind: TzObservanceKind,
        tz_prop: TzPropBuilder,
    ) -> ParseResult<()> {
        match self {
            Self::Timezone(b) => Ok(match kind {
                TzObservanceKind::Standard => b.standardc.push(tz_prop),
                TzObservanceKind::Daylight => b.daylightc.push(tz_prop),
            }),
            _ => Err(ParseError::UnexpectedComponent("STANDARD/DAYLIGHT")),
        }
    }

    /// Builds this builder into its finished [`CalComponent`], validating
    /// whatever couldn't be checked property-by-property during ingest.
    /// `has_method` is only consulted by [`EventBuilder::build`] (RFC 5545
    /// §3.6.1's `DTSTART`/`METHOD` interaction) — every other component
    /// ignores it.
    fn build(self, has_method: bool) -> Result<CalComponent, ComponentError> {
        Ok(match self {
            Self::Event(b) => CalComponent::Event(b.build(has_method)?),
            Self::Todo(b) => CalComponent::Todo(b.build()?),
            Self::Journal(b) => CalComponent::Journal(b.build()?),
            Self::FreeBusy(b) => CalComponent::FreeBusy(b.build()?),
            Self::Timezone(b) => CalComponent::Timezone(b.build()?),
            Self::Unknown(b) => CalComponent::Unknown(b.build()),
        })
    }
}

/// Routes one already-parsed [`Property`] into a component builder's own
/// fields. Implemented once per builder, each owning the decision of what's
/// legal for its own component type (RFC 5545 §3.6) — a name absent from a
/// given impl's `match` isn't valid on that component and is an error, the
/// same way [`Property::parse`] owns "what does this keyword mean" instead
/// of the parser.
trait PropertyIngest {
    fn ingest(&mut self, p: Property) -> ParseResult<()>;
}

/// Assigns `value` into a singleton property slot, or reports the RFC 5545
/// violation of the same property occurring twice in one component.
fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    name: &'static str,
) -> ParseResult<()> {
    if slot.is_some() {
        return Err(PropertyError::DuplicateProperty(name).into());
    }
    *slot = Some(value);
    Ok(())
}

/// Cross-field comparisons between a component's `DTSTART` and its sibling
/// properties. Deferred to `build()` (per this crate's validation
/// architecture — see `CLAUDE.md`), once every property on the component is
/// known, rather than living on `DateTimeStart` itself or on the sibling
/// type, neither of which has access to the other at parse time.
impl DateTimeStart {
    /// RFC 5545 §3.8.2.2/§3.8.2.3: `DTEND`'s and `DUE`'s value type "MUST be
    /// the same value type as the 'DTSTART' property". `name` is used only
    /// for the error message, so this doubles as the implementation of
    /// [`Self::cmp_until`].
    fn cmp_value_type(
        &self,
        other: Option<&DateOrDatetime>,
        name: &'static str,
    ) -> Result<(), ComponentError> {
        let Some(other) = other else {
            return Ok(());
        };
        let matches_type = matches!(
            (self.value(), other),
            (DateOrDatetime::Date(_), DateOrDatetime::Date(_))
                | (DateOrDatetime::DateTime(_), DateOrDatetime::DateTime(_))
        );
        if matches_type {
            Ok(())
        } else {
            Err(ComponentError::MismatchedValueType(name, "DTSTART"))
        }
    }

    /// RFC 5545 §3.3.10: `RRULE`'s `UNTIL` rule part "MUST have the same
    /// value type as the 'DTSTART' property". Checked here, once both are
    /// known, rather than in `Recur::try_from` — which parses `RRULE` on its
    /// own and has no access to the sibling `DTSTART`.
    fn cmp_until(&self, rrule: Option<&RRule>) -> Result<(), ComponentError> {
        let Some(rrule) = rrule else {
            return Ok(());
        };
        let Some(until) = rrule.recur().until() else {
            return Ok(());
        };
        self.cmp_value_type(Some(until), "RRULE's UNTIL")
    }

    /// RFC 5545 §3.8.5.1: "The value type of this property MUST be the same
    /// as the value type of the 'DTSTART' property" — checked once per
    /// `EXDATE` value listed across every `EXDATE` property on the
    /// component (`EXDATE` takes a comma-separated list, and a component
    /// MAY repeat the property).
    fn cmp_exdate(
        &self,
        exdate: &[ExceptionDateTimes],
    ) -> Result<(), ComponentError> {
        let matches_type = exdate
            .iter()
            .flat_map(ExceptionDateTimes::value)
            .all(|value| {
                matches!(
                    (self.value(), value),
                    (DateOrDatetime::Date(_), DateOrDatetime::Date(_))
                        | (
                            DateOrDatetime::DateTime(_),
                            DateOrDatetime::DateTime(_)
                        )
                )
            });
        if matches_type {
            Ok(())
        } else {
            Err(ComponentError::MismatchedValueType("EXDATE", "DTSTART"))
        }
    }

    /// RFC 5545 §3.8.5.2: "The value type of the 'RDATE' property, if
    /// specified, MUST be the same as the 'DTSTART' property, or its value
    /// type must be PERIOD" — a `PERIOD` value is always allowed regardless
    /// of `DTSTART`'s value type, unlike `EXDATE`, which has no `PERIOD`
    /// alternative.
    fn cmp_rdate(
        &self,
        rdate: &[RecurrenceDateTimes],
    ) -> Result<(), ComponentError> {
        let matches_type = rdate
            .iter()
            .flat_map(RecurrenceDateTimes::value)
            .all(|value| {
                matches!(
                    (self.value(), value),
                    (DateOrDatetime::Date(_), DateTimePeriod::Date(_))
                        | (
                            DateOrDatetime::DateTime(_),
                            DateTimePeriod::DateTime(_)
                        )
                        | (_, DateTimePeriod::Period(_))
                )
            });
        if matches_type {
            Ok(())
        } else {
            Err(ComponentError::MismatchedValueType("RDATE", "DTSTART"))
        }
    }

    /// RFC 5545 §3.8.5.1 requires `EXDATE`'s value type to match
    /// `DTSTART`'s; real-world producers extend that to expecting the same
    /// `TZID` too (see issue #6) — a `DTSTART;TZID=America/New_York` paired
    /// with an `EXDATE;TZID=Europe/London` value (or one specifying no
    /// `TZID` at all) names a different wall-clock instant than intended,
    /// even though both are DATE-TIME. Checked once per `EXDATE` property
    /// occurrence (the `TZID` parameter applies once to the whole
    /// comma-separated value list).
    fn cmp_exdate_tzid(
        &self,
        exdate: &[ExceptionDateTimes],
    ) -> Result<(), ComponentError> {
        let matches_tzid = exdate.iter().all(|e| e.tzid() == self.tzid());
        if matches_tzid {
            Ok(())
        } else {
            Err(ComponentError::MismatchedTzid("EXDATE", "DTSTART"))
        }
    }

    /// RFC 5545 §3.8.5.2 requires `RDATE`'s value type to match `DTSTART`'s
    /// (or be `PERIOD`); real-world producers extend that to expecting the
    /// same `TZID` too when both are DATE-TIME (see [`Self::cmp_exdate_tzid`]).
    fn cmp_rdate_tzid(
        &self,
        rdate: &[RecurrenceDateTimes],
    ) -> Result<(), ComponentError> {
        let matches_tzid = rdate.iter().all(|r| r.tzid() == self.tzid());
        if matches_tzid {
            Ok(())
        } else {
            Err(ComponentError::MismatchedTzid("RDATE", "DTSTART"))
        }
    }
}

/// A calendar property, wrapping every property type defined in
/// [`crate::properties`].
#[derive(Debug)]
pub(crate) enum Property {
    /// `CALSCALE` ([`CalendarScale`]).
    CalendarScale(CalendarScale),
    /// `METHOD` ([`Method`]).
    Method(Method),
    /// `PRODID` ([`ProductIdentifier`]).
    ProductIdentifier(ProductIdentifier),
    /// `VERSION` ([`Version`]).
    Version(Version),
    /// `ACTION` ([`Action`]).
    Action(Action),
    /// `REPEAT` ([`Repeat`]).
    Repeat(Repeat),
    /// `TRIGGER` ([`Trigger`]).
    Trigger(Trigger),
    /// `CREATED` ([`DateTimeCreated`]).
    DateTimeCreated(DateTimeCreated),
    /// `DTSTAMP` ([`DateTimeStamp`]).
    DateTimeStamp(DateTimeStamp),
    /// `LAST-MODIFIED` ([`LastModified`]).
    LastModified(LastModified),
    /// `SEQUENCE` ([`Sequence`]).
    Sequence(Sequence),
    /// `COMPLETED` ([`Completed`]).
    Completed(Completed),
    /// `DTEND` ([`DateTimeEnd`]).
    DateTimeEnd(DateTimeEnd),
    /// `DUE` ([`DateTimeDue`]).
    DateTimeDue(DateTimeDue),
    /// `DTSTART` ([`DateTimeStart`]).
    DateTimeStart(DateTimeStart),
    /// `DURATION` ([`Duration`]).
    Duration(Duration),
    /// `FREEBUSY` ([`FreeBusyTime`]).
    FreeBusyTime(FreeBusyTime),
    /// `TRANSP` ([`TimeTransparency`]).
    TimeTransparency(TimeTransparency),
    /// `REQUEST-STATUS` ([`RequestStatus`]).
    RequestStatus(RequestStatus),
    /// `ATTACH` ([`Attachment`]).
    Attachment(Attachment),
    /// `CATEGORIES` ([`Categories`]).
    Categories(Categories),
    /// `CLASS` ([`Classification`]).
    Classification(Classification),
    /// `COMMENT` ([`Comment`]).
    Comment(Comment),
    /// `DESCRIPTION` ([`Description`]).
    Description(Description),
    /// `GEO` ([`Geo`]).
    Geo(Geo),
    /// `LOCATION` ([`Location`]).
    Location(Location),
    /// `PERCENT-COMPLETE` ([`PercentComplete`]).
    PercentComplete(PercentComplete),
    /// `PRIORITY` ([`Priority`]).
    Priority(Priority),
    /// `RESOURCES` ([`Resources`]).
    Resources(Resources),
    /// `STATUS` ([`Status`]).
    Status(Status),
    /// `SUMMARY` ([`Summary`]).
    Summary(Summary),
    /// `ATTENDEE` ([`Attendee`]).
    Attendee(Attendee),
    /// `CONTACT` ([`Contact`]).
    Contact(Contact),
    /// `ORGANIZER` ([`Organizer`]).
    Organizer(Organizer),
    /// `RECURRENCE-ID` ([`RecurrenceId`]).
    RecurrenceId(RecurrenceId),
    /// `RELATED-TO` ([`RelatedTo`]).
    RelatedTo(RelatedTo),
    /// `URL` ([`UniformResourceLocator`]).
    UniformResourceLocator(UniformResourceLocator),
    /// `UID` ([`Uid`]).
    Uid(Uid),
    /// `TZID` ([`TimeZoneIdentifier`]).
    TimeZoneIdentifier(TimeZoneIdentifier),
    /// `TZNAME` ([`TimeZoneName`]).
    TimeZoneName(TimeZoneName),
    /// `TZOFFSETFROM` ([`TimeZoneOffsetFrom`]).
    TimeZoneOffsetFrom(TimeZoneOffsetFrom),
    /// `TZOFFSETTO` ([`TimeZoneOffsetTo`]).
    TimeZoneOffsetTo(TimeZoneOffsetTo),
    /// `TZURL` ([`TimeZoneUrl`]).
    TimeZoneUrl(TimeZoneUrl),
    /// `EXDATE` ([`ExceptionDateTimes`]).
    ExceptionDateTimes(ExceptionDateTimes),
    /// `RDATE` ([`RecurrenceDateTimes`]).
    RecurrenceDateTimes(RecurrenceDateTimes),
    /// `RRULE` ([`RRule`]).
    RRule(RRule),
    /// A non-standard, experimental property (`X-` prefixed).
    Xprop(Xprop),
    /// An IANA-registered property.
    Iana(Iana),
}

impl From<CalendarScale> for Property {
    fn from(value: CalendarScale) -> Self {
        Self::CalendarScale(value)
    }
}
impl From<Method> for Property {
    fn from(value: Method) -> Self {
        Self::Method(value)
    }
}
impl From<ProductIdentifier> for Property {
    fn from(value: ProductIdentifier) -> Self {
        Self::ProductIdentifier(value)
    }
}
impl From<Version> for Property {
    fn from(value: Version) -> Self {
        Self::Version(value)
    }
}
impl From<Action> for Property {
    fn from(value: Action) -> Self {
        Self::Action(value)
    }
}
impl From<Repeat> for Property {
    fn from(value: Repeat) -> Self {
        Self::Repeat(value)
    }
}
impl From<Trigger> for Property {
    fn from(value: Trigger) -> Self {
        Self::Trigger(value)
    }
}
impl From<DateTimeCreated> for Property {
    fn from(value: DateTimeCreated) -> Self {
        Self::DateTimeCreated(value)
    }
}
impl From<DateTimeStamp> for Property {
    fn from(value: DateTimeStamp) -> Self {
        Self::DateTimeStamp(value)
    }
}
impl From<LastModified> for Property {
    fn from(value: LastModified) -> Self {
        Self::LastModified(value)
    }
}
impl From<Sequence> for Property {
    fn from(value: Sequence) -> Self {
        Self::Sequence(value)
    }
}
impl From<Completed> for Property {
    fn from(value: Completed) -> Self {
        Self::Completed(value)
    }
}
impl From<DateTimeEnd> for Property {
    fn from(value: DateTimeEnd) -> Self {
        Self::DateTimeEnd(value)
    }
}
impl From<DateTimeDue> for Property {
    fn from(value: DateTimeDue) -> Self {
        Self::DateTimeDue(value)
    }
}
impl From<DateTimeStart> for Property {
    fn from(value: DateTimeStart) -> Self {
        Self::DateTimeStart(value)
    }
}
impl From<Duration> for Property {
    fn from(value: Duration) -> Self {
        Self::Duration(value)
    }
}
impl From<FreeBusyTime> for Property {
    fn from(value: FreeBusyTime) -> Self {
        Self::FreeBusyTime(value)
    }
}
impl From<TimeTransparency> for Property {
    fn from(value: TimeTransparency) -> Self {
        Self::TimeTransparency(value)
    }
}
impl From<RequestStatus> for Property {
    fn from(value: RequestStatus) -> Self {
        Self::RequestStatus(value)
    }
}
impl From<Attachment> for Property {
    fn from(value: Attachment) -> Self {
        Self::Attachment(value)
    }
}
impl From<Categories> for Property {
    fn from(value: Categories) -> Self {
        Self::Categories(value)
    }
}
impl From<Classification> for Property {
    fn from(value: Classification) -> Self {
        Self::Classification(value)
    }
}
impl From<Comment> for Property {
    fn from(value: Comment) -> Self {
        Self::Comment(value)
    }
}
impl From<Description> for Property {
    fn from(value: Description) -> Self {
        Self::Description(value)
    }
}
impl From<Geo> for Property {
    fn from(value: Geo) -> Self {
        Self::Geo(value)
    }
}
impl From<Location> for Property {
    fn from(value: Location) -> Self {
        Self::Location(value)
    }
}
impl From<PercentComplete> for Property {
    fn from(value: PercentComplete) -> Self {
        Self::PercentComplete(value)
    }
}
impl From<Priority> for Property {
    fn from(value: Priority) -> Self {
        Self::Priority(value)
    }
}
impl From<Resources> for Property {
    fn from(value: Resources) -> Self {
        Self::Resources(value)
    }
}
impl From<Status> for Property {
    fn from(value: Status) -> Self {
        Self::Status(value)
    }
}
impl From<Summary> for Property {
    fn from(value: Summary) -> Self {
        Self::Summary(value)
    }
}
impl From<Attendee> for Property {
    fn from(value: Attendee) -> Self {
        Self::Attendee(value)
    }
}
impl From<Contact> for Property {
    fn from(value: Contact) -> Self {
        Self::Contact(value)
    }
}
impl From<Organizer> for Property {
    fn from(value: Organizer) -> Self {
        Self::Organizer(value)
    }
}
impl From<RecurrenceId> for Property {
    fn from(value: RecurrenceId) -> Self {
        Self::RecurrenceId(value)
    }
}
impl From<RelatedTo> for Property {
    fn from(value: RelatedTo) -> Self {
        Self::RelatedTo(value)
    }
}
impl From<UniformResourceLocator> for Property {
    fn from(value: UniformResourceLocator) -> Self {
        Self::UniformResourceLocator(value)
    }
}
impl From<Uid> for Property {
    fn from(value: Uid) -> Self {
        Self::Uid(value)
    }
}
impl From<TimeZoneIdentifier> for Property {
    fn from(value: TimeZoneIdentifier) -> Self {
        Self::TimeZoneIdentifier(value)
    }
}
impl From<TimeZoneName> for Property {
    fn from(value: TimeZoneName) -> Self {
        Self::TimeZoneName(value)
    }
}
impl From<TimeZoneOffsetFrom> for Property {
    fn from(value: TimeZoneOffsetFrom) -> Self {
        Self::TimeZoneOffsetFrom(value)
    }
}
impl From<TimeZoneOffsetTo> for Property {
    fn from(value: TimeZoneOffsetTo) -> Self {
        Self::TimeZoneOffsetTo(value)
    }
}
impl From<TimeZoneUrl> for Property {
    fn from(value: TimeZoneUrl) -> Self {
        Self::TimeZoneUrl(value)
    }
}
impl From<ExceptionDateTimes> for Property {
    fn from(value: ExceptionDateTimes) -> Self {
        Self::ExceptionDateTimes(value)
    }
}
impl From<RecurrenceDateTimes> for Property {
    fn from(value: RecurrenceDateTimes) -> Self {
        Self::RecurrenceDateTimes(value)
    }
}
impl From<RRule> for Property {
    fn from(value: RRule) -> Self {
        Self::RRule(value)
    }
}
impl From<Xprop> for Property {
    fn from(value: Xprop) -> Self {
        Self::Xprop(value)
    }
}
impl From<Iana> for Property {
    fn from(value: Iana) -> Self {
        Self::Iana(value)
    }
}

/// Parses a property's raw, unparsed remainder (`*(";" param) ":" value`,
/// exactly what a
/// [`TokenType::Property`](super::ast::token::TokenType::Property)
/// token's `literal()` carries) into the matching [`Property`] variant.
/// Backs [`Property::parse`]'s dispatch table.
type PropertyParser = fn(&[u8]) -> ParseResult<Property>;

/// Name -> parser dispatch table for every property RFC 5545 defines by
/// keyword (§3.7, §3.8), used by [`Property::parse`]. This is the single
/// place that knows "PRODID means a `ProductIdentifier`" — replacing what
/// used to be a `TokenType` keyword classified by the lexer itself. Built
/// with [`phf`] (the same mechanism the old lexer used for its `KEYWORDS`
/// map) so the lookup stays O(1) — a compile-time perfect hash, not a
/// `match` over byte-string patterns (which codegens as a comparison chain,
/// not a jump table).
static PROPERTY_DISPATCH: phf::Map<&'static [u8], PropertyParser> = phf::phf_map! {
    b"CALSCALE" => |v| CalendarScale::try_from(v).map(Into::into),
    b"METHOD" => |v| Method::try_from(v).map(Into::into),
    b"PRODID" => |v| ProductIdentifier::try_from(v).map(Into::into),
    b"VERSION" => |v| Version::try_from(v).map(Into::into),
    b"ACTION" => |v| Action::try_from(v).map(Into::into),
    b"REPEAT" => |v| Repeat::try_from(v).map(Into::into),
    b"TRIGGER" => |v| Trigger::try_from(v).map(Into::into),
    b"CREATED" => |v| DateTimeCreated::try_from(v).map(Into::into),
    b"DTSTAMP" => |v| DateTimeStamp::try_from(v).map(Into::into),
    b"LAST-MODIFIED" => |v| LastModified::try_from(v).map(Into::into),
    b"SEQUENCE" => |v| Sequence::try_from(v).map(Into::into),
    b"COMPLETED" => |v| Completed::try_from(v).map(Into::into),
    b"DTEND" => |v| DateTimeEnd::try_from(v).map(Into::into),
    b"DUE" => |v| DateTimeDue::try_from(v).map(Into::into),
    b"DTSTART" => |v| DateTimeStart::try_from(v).map(Into::into),
    b"DURATION" => |v| Duration::try_from(v).map(Into::into),
    b"FREEBUSY" => |v| FreeBusyTime::try_from(v).map(Into::into),
    b"TRANSP" => |v| TimeTransparency::try_from(v).map(Into::into),
    b"REQUEST-STATUS" => |v| RequestStatus::try_from(v).map(Into::into),
    b"ATTACH" => |v| Attachment::try_from(v).map(Into::into),
    b"CATEGORIES" => |v| Categories::try_from(v).map(Into::into),
    b"CLASS" => |v| Classification::try_from(v).map(Into::into),
    b"COMMENT" => |v| Comment::try_from(v).map(Into::into),
    b"DESCRIPTION" => |v| Description::try_from(v).map(Into::into),
    b"GEO" => |v| Geo::try_from(v).map(Into::into),
    b"LOCATION" => |v| Location::try_from(v).map(Into::into),
    b"PERCENT-COMPLETE" => |v| PercentComplete::try_from(v).map(Into::into),
    b"PRIORITY" => |v| Priority::try_from(v).map(Into::into),
    b"RESOURCES" => |v| Resources::try_from(v).map(Into::into),
    b"STATUS" => |v| Status::try_from(v).map(Into::into),
    b"SUMMARY" => |v| Summary::try_from(v).map(Into::into),
    b"ATTENDEE" => |v| Attendee::try_from(v).map(Into::into),
    b"CONTACT" => |v| Contact::try_from(v).map(Into::into),
    b"ORGANIZER" => |v| Organizer::try_from(v).map(Into::into),
    b"RECURRENCE-ID" => |v| RecurrenceId::try_from(v).map(Into::into),
    b"RELATED-TO" => |v| RelatedTo::try_from(v).map(Into::into),
    b"URL" => |v| UniformResourceLocator::try_from(v).map(Into::into),
    b"UID" => |v| Uid::try_from(v).map(Into::into),
    b"TZID" => |v| TimeZoneIdentifier::try_from(v).map(Into::into),
    b"TZNAME" => |v| TimeZoneName::try_from(v).map(Into::into),
    b"TZOFFSETFROM" => |v| TimeZoneOffsetFrom::try_from(v).map(Into::into),
    b"TZOFFSETTO" => |v| TimeZoneOffsetTo::try_from(v).map(Into::into),
    b"TZURL" => |v| TimeZoneUrl::try_from(v).map(Into::into),
    b"EXDATE" => |v| ExceptionDateTimes::try_from(v).map(Into::into),
    b"RDATE" => |v| RecurrenceDateTimes::try_from(v).map(Into::into),
    b"RRULE" => |v| RRule::try_from(v).map(Into::into),
};

impl Property {
    /// Parses a [`TokenType::Property`](super::ast::token::TokenType::Property)
    /// token's name and raw remainder (`*(";" param) ":" value`) into the
    /// matching [`Property`] variant, dispatching on the token's upper-cased
    /// name (`lexeme()`) via [`PROPERTY_DISPATCH`]. A name absent from the
    /// table isn't an error — `X-`/IANA extension properties are open-ended
    /// by design
    /// ([Section 3.8.8](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.8))
    /// — it just falls back to [`Xprop`]/[`Iana`].
    pub(crate) fn parse(
        name: &[u8],
        remainder: &[u8],
    ) -> ParseResult<Property> {
        if let Some(parse) = PROPERTY_DISPATCH.get(name) {
            parse(remainder)
        } else if name.starts_with(b"X-") {
            Xprop::try_from(remainder).map(Into::into)
        } else {
            Iana::try_from(remainder).map(Into::into)
        }
    }
}

#[derive(Default, Debug)]
struct CalendarBuilder {
    prodid: Option<ProductIdentifier>,
    version: Option<Version>,
    calscale: Option<CalendarScale>,
    method: Option<Method>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
    components: Vec<Component>,
}

impl CalendarBuilder {
    /// creates a new cal builder
    fn new() -> Self {
        Self::default()
    }

    fn build(self) -> Result<Calendar, ComponentError> {
        let prodid =
            self.prodid.ok_or(ComponentError::MissingField("PRODID"))?;
        let version = self
            .version
            .ok_or(ComponentError::MissingField("VERSION"))?;
        if self.components.is_empty() {
            return Err(ComponentError::RequiresAtLeastOne(
                "VCALENDAR",
                "calendar component",
            ));
        }
        // DTSTART is only REQUIRED on a VEVENT when METHOD is absent (RFC
        // 5545 §3.6.1) — every component needs to know this to validate
        // itself, so it's threaded down rather than re-checked per-event.
        let has_method = self.method.is_some();
        let components = self
            .components
            .into_iter()
            .map(|c| c.build(has_method))
            .collect::<Result<Vec<_>, _>>()?;
        validate_timezones(&components)?;
        validate_no_duplicate_uid(&components)?;

        Ok(Calendar {
            prodid,
            version,
            calscale: self.calscale,
            method: self.method,
            xprop: self.xprop,
            iana: self.iana,
            components,
        })
    }
}

/// RFC 5545 §3.6.5 also says multiple `VTIMEZONE`s "MUST represent a unique
/// time zone definition" each — checked here since it's calendar-wide (needs
/// every `VTIMEZONE` in the object), so this runs once in
/// `CalendarBuilder::build`, after every component has already been
/// individually validated and built.
///
/// This function does *not* enforce the RFC's related requirement that "An
/// individual 'VTIMEZONE' calendar component MUST be specified for each
/// unique 'TZID' parameter value" — every `TZID` parameter
/// that exists as a [`TzIdParam`] has, by construction, already been
/// resolved against `chrono_tz`'s own IANA database (`TzIdParam::try_from`
/// fails otherwise), which is a real, accurate source of offset rules
/// independent of any local `VTIMEZONE`. Requiring a matching `VTIMEZONE`
/// on top of that would reject the majority of real-world calendars, which
/// reference well-known zones like `America/New_York` without also
/// embedding a `VTIMEZONE` for them.
fn validate_timezones(
    components: &[CalComponent],
) -> Result<(), ComponentError> {
    let mut declared = HashSet::new();
    for c in components {
        if let CalComponent::Timezone(tz) = c {
            let name = tz.tzid.as_str();
            if !declared.insert(name) {
                return Err(ComponentError::DuplicateTimeZone(name.into()));
            }
        }
    }
    Ok(())
}

/// RFC 5545 doesn't require a `UID` to be unique within one `VCALENDAR` —
/// a recurring master and its `RECURRENCE-ID`-bearing overrides
/// legitimately share one. What can't happen is two components claiming
/// the exact same instance: identical `UID` and identical (including both
/// absent) `RECURRENCE-ID`. Only `VEVENT`/`VTODO`/`VJOURNAL` participate in
/// this identity model; `VFREEBUSY`'s `UID` identifies a request/reply, not
/// a recurring instance, so it's excluded.
fn validate_no_duplicate_uid(
    components: &[CalComponent],
) -> Result<(), ComponentError> {
    let mut seen: Vec<(&str, Option<&DateOrDatetime>)> = Vec::new();
    for c in components {
        let (uid, recurid) = match c {
            CalComponent::Event(e) => {
                (e.uid.as_str(), e.recurid.as_ref().map(RecurrenceId::value))
            }
            CalComponent::Todo(t) => {
                (t.uid.as_str(), t.recur_id.as_ref().map(RecurrenceId::value))
            }
            CalComponent::Journal(j) => {
                (j.uid.as_str(), j.recurid.as_ref().map(RecurrenceId::value))
            }
            CalComponent::FreeBusy(_)
            | CalComponent::Timezone(_)
            | CalComponent::Unknown(_) => continue,
        };
        if seen.contains(&(uid, recurid)) {
            return Err(ComponentError::DuplicateUid(uid.into()));
        }
        seen.push((uid, recurid));
    }
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum ComponentError {
    #[error("Missing field: {0}")]
    MissingField(&'static str),

    /// Two properties that RFC 5545 says MUST NOT both appear in the same
    /// component (e.g. `DTEND`/`DURATION`) were both present.
    #[error("{0} and {1} MUST NOT both be specified in the same component")]
    MutuallyExclusive(&'static str, &'static str),

    /// A property that RFC 5545 only allows conditional on another being
    /// present (e.g. `DURATION` requiring `DTSTART` in a `VTODO`) showed up
    /// without it.
    #[error("{0} MUST NOT be specified without {1}")]
    Requires(&'static str, &'static str),

    /// Two properties that RFC 5545 says MUST occur together or not at all
    /// (e.g. `DURATION`/`REPEAT` in a `VALARM`) had only one present.
    #[error("{0} and {1} MUST occur together or not at all")]
    RequiresTogether(&'static str, &'static str),

    /// A component/property required at least one of some other property,
    /// but none were present (e.g. `VALARM` with `ACTION:EMAIL` requires at
    /// least one `ATTENDEE`).
    #[error("{0} requires at least one {1}")]
    RequiresAtLeastOne(&'static str, &'static str),

    /// A property RFC 5545 disallows for a particular alternative grammar
    /// showed up anyway (e.g. `ATTACH` in a `VALARM` with `ACTION:DISPLAY`).
    #[error("{0} MUST NOT be specified when {1}")]
    NotAllowed(&'static str, &'static str),

    /// Two properties whose value types RFC 5545 requires to match (e.g.
    /// `RRULE`'s `UNTIL` and the component's `DTSTART`) didn't.
    #[error("{0}'s value type MUST match {1}'s (both DATE, or both DATE-TIME)")]
    MismatchedValueType(&'static str, &'static str),

    /// `EXDATE`/`RDATE`'s `TZID` parameter, when either side specifies one,
    /// didn't match the component's `DTSTART` (RFC 5545 §3.8.5.1/§3.8.5.2
    /// require the value *type* to match; real-world producers extend that
    /// to the zone itself — see issue #6).
    #[error("{0}'s TZID MUST match {1}'s TZID")]
    MismatchedTzid(&'static str, &'static str),

    /// The same `TZID` was defined by more than one `VTIMEZONE` component in
    /// one `VCALENDAR` (RFC 5545 §3.6.5: "an individual VTIMEZONE...MUST be
    /// specified for each unique TZID").
    #[error("TZID={0} is defined by more than one VTIMEZONE in this VCALENDAR")]
    DuplicateTimeZone(String),

    /// Two components in the same `VCALENDAR` share both `UID` and (absent
    /// or identical) `RECURRENCE-ID` — RFC 5545 doesn't forbid reusing a
    /// `UID` across a recurring master and its overrides, but two
    /// components claiming the very same instance (or both claiming to be
    /// the master) can't both be right.
    #[error(
        "UID={0} is shared by more than one component with the same \
         RECURRENCE-ID"
    )]
    DuplicateUid(String),
}

#[derive(Debug, Default)]
struct EventBuilder {
    dtstamp: Option<DateTimeStamp>,
    uid: Option<Uid>,
    /// The following is REQUIRED if the component
    /// appears in an iCalendar object that doesn't
    /// specify the "METHOD" property; otherwise, it
    /// is OPTIONAL; in any case, it MUST NOT occur
    /// more than once.
    dtstart: Option<DateTimeStart>,
    class: Option<Classification>,
    created: Option<DateTimeCreated>,
    description: Option<Description>,
    geo: Option<Geo>,
    last_mod: Option<LastModified>,
    location: Option<Location>,
    organizer: Option<Organizer>,
    priority: Option<Priority>,
    seq: Option<Sequence>,
    status: Option<Status>,
    summary: Option<Summary>,
    transp: Option<TimeTransparency>,
    url: Option<UniformResourceLocator>,
    recurid: Option<RecurrenceId>,
    rrule: Option<RRule>,
    dtend: Option<DateTimeEnd>,
    duration: Option<Duration>,
    attach: Vec<Attachment>,
    attendee: Vec<Attendee>,
    categories: Vec<Categories>,
    comment: Vec<Comment>,
    contact: Vec<Contact>,
    exdate: Vec<ExceptionDateTimes>,
    rstatus: Vec<RequestStatus>,
    related: Vec<RelatedTo>,
    resources: Vec<Resources>,
    rdate: Vec<RecurrenceDateTimes>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
    alarms: Vec<AlarmBuilder>,
}

impl EventBuilder {
    fn new() -> Self {
        Self::default()
    }

    /// Validates the cross-field rules RFC 5545 §3.6.1 places on `VEVENT`
    /// and assembles the finished [`Event`]. `has_method` is whether the
    /// enclosing `VCALENDAR` specified a `METHOD` property — `DTSTART` is
    /// only REQUIRED here when it didn't.
    fn build(self, has_method: bool) -> Result<Event, ComponentError> {
        if !has_method && self.dtstart.is_none() {
            return Err(ComponentError::MissingField("DTSTART"));
        }
        if self.dtend.is_some() && self.duration.is_some() {
            return Err(ComponentError::MutuallyExclusive("DTEND", "DURATION"));
        }
        if let Some(dtstart) = self.dtstart.as_ref() {
            dtstart.cmp_until(self.rrule.as_ref())?;
            dtstart.cmp_value_type(
                self.dtend.as_ref().map(DateTimeEnd::value),
                "DTEND",
            )?;
            dtstart.cmp_exdate(&self.exdate)?;
            dtstart.cmp_rdate(&self.rdate)?;
            dtstart.cmp_exdate_tzid(&self.exdate)?;
            dtstart.cmp_rdate_tzid(&self.rdate)?;
        }
        let alarms = self
            .alarms
            .into_iter()
            .map(AlarmBuilder::build)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Event {
            dtstamp: self
                .dtstamp
                .ok_or(ComponentError::MissingField("DTSTAMP"))?,
            uid: self.uid.ok_or(ComponentError::MissingField("UID"))?,
            dtstart: self.dtstart,
            class: self.class,
            created: self.created,
            description: self.description,
            geo: self.geo,
            last_mod: self.last_mod,
            location: self.location,
            organizer: self.organizer,
            priority: self.priority,
            seq: self.seq,
            status: self.status,
            summary: self.summary,
            transp: self.transp,
            url: self.url,
            recurid: self.recurid,
            rrule: self.rrule,
            dtend: self.dtend,
            duration: self.duration,
            attach: self.attach,
            attendee: self.attendee,
            categories: self.categories,
            comment: self.comment,
            contact: self.contact,
            exdate: self.exdate,
            rstatus: self.rstatus,
            related: self.related,
            resources: self.resources,
            rdate: self.rdate,
            xprop: self.xprop,
            iana: self.iana,
            alarms,
        })
    }
}

impl PropertyIngest for EventBuilder {
    fn ingest(&mut self, p: Property) -> ParseResult<()> {
        match p {
            Property::DateTimeStamp(v) => {
                set_once(&mut self.dtstamp, v, "DTSTAMP")
            }
            Property::Uid(v) => set_once(&mut self.uid, v, "UID"),
            Property::DateTimeStart(v) => {
                set_once(&mut self.dtstart, v, "DTSTART")
            }
            Property::Classification(v) => {
                set_once(&mut self.class, v, "CLASS")
            }
            Property::DateTimeCreated(v) => {
                set_once(&mut self.created, v, "CREATED")
            }
            Property::Description(v) => {
                set_once(&mut self.description, v, "DESCRIPTION")
            }
            Property::Geo(v) => set_once(&mut self.geo, v, "GEO"),
            Property::LastModified(v) => {
                set_once(&mut self.last_mod, v, "LAST-MODIFIED")
            }
            Property::Location(v) => {
                set_once(&mut self.location, v, "LOCATION")
            }
            Property::Organizer(v) => {
                set_once(&mut self.organizer, v, "ORGANIZER")
            }
            Property::Priority(v) => {
                set_once(&mut self.priority, v, "PRIORITY")
            }
            Property::Sequence(v) => set_once(&mut self.seq, v, "SEQUENCE"),
            Property::Status(v) => set_once(&mut self.status, v, "STATUS"),
            Property::Summary(v) => set_once(&mut self.summary, v, "SUMMARY"),
            Property::TimeTransparency(v) => {
                set_once(&mut self.transp, v, "TRANSP")
            }
            Property::UniformResourceLocator(v) => {
                set_once(&mut self.url, v, "URL")
            }
            Property::RecurrenceId(v) => {
                set_once(&mut self.recurid, v, "RECURRENCE-ID")
            }
            Property::RRule(v) => set_once(&mut self.rrule, v, "RRULE"),
            // DTEND and DURATION are mutually exclusive within a VEVENT
            // (RFC 5545 §3.6.1) — that's a cross-field rule, checked in
            // `build()` once every property has been seen, not here.
            Property::DateTimeEnd(v) => set_once(&mut self.dtend, v, "DTEND"),
            Property::Duration(v) => {
                set_once(&mut self.duration, v, "DURATION")
            }
            Property::Attachment(v) => Ok(self.attach.push(v)),
            Property::Attendee(v) => Ok(self.attendee.push(v)),
            Property::Categories(v) => Ok(self.categories.push(v)),
            Property::Comment(v) => Ok(self.comment.push(v)),
            Property::Contact(v) => Ok(self.contact.push(v)),
            Property::ExceptionDateTimes(v) => Ok(self.exdate.push(v)),
            Property::RequestStatus(v) => Ok(self.rstatus.push(v)),
            Property::RelatedTo(v) => Ok(self.related.push(v)),
            Property::Resources(v) => Ok(self.resources.push(v)),
            Property::RecurrenceDateTimes(v) => Ok(self.rdate.push(v)),
            Property::Xprop(v) => Ok(self.xprop.push(v)),
            Property::Iana(v) => Ok(self.iana.push(v)),
            _ => Err(PropertyError::UnexpectedProperty.into()),
        }
    }
}

#[derive(Debug, Default)]
struct TodoBuilder {
    dtstamp: Option<DateTimeStamp>,
    uid: Option<Uid>,
    class: Option<Classification>,
    completed: Option<Completed>,
    created: Option<DateTimeCreated>,
    description: Option<Description>,
    dtstart: Option<DateTimeStart>,
    geo: Option<Geo>,
    last_mod: Option<LastModified>,
    location: Option<Location>,
    organizer: Option<Organizer>,
    percent: Option<PercentComplete>,
    priority: Option<Priority>,
    recur_id: Option<RecurrenceId>,
    seq: Option<Sequence>,
    status: Option<Status>,
    summary: Option<Summary>,
    url: Option<UniformResourceLocator>,
    rrule: Option<RRule>,
    due: Option<DateTimeDue>,
    duration: Option<Duration>,
    attach: Vec<Attachment>,
    attendee: Vec<Attendee>,
    categories: Vec<Categories>,
    comment: Vec<Comment>,
    contact: Vec<Contact>,
    exdate: Vec<ExceptionDateTimes>,
    rstatus: Vec<RequestStatus>,
    related: Vec<RelatedTo>,
    resources: Vec<Resources>,
    rdate: Vec<RecurrenceDateTimes>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
    alarms: Vec<AlarmBuilder>,
}

impl TodoBuilder {
    fn new() -> Self {
        Self::default()
    }

    /// Validates the cross-field rules RFC 5545 §3.6.2 places on `VTODO`
    /// and assembles the finished [`Todo`].
    fn build(self) -> Result<Todo, ComponentError> {
        if self.due.is_some() && self.duration.is_some() {
            return Err(ComponentError::MutuallyExclusive("DUE", "DURATION"));
        }
        if self.duration.is_some() && self.dtstart.is_none() {
            return Err(ComponentError::Requires("DURATION", "DTSTART"));
        }
        if let Some(dtstart) = self.dtstart.as_ref() {
            dtstart.cmp_until(self.rrule.as_ref())?;
            dtstart.cmp_value_type(
                self.due.as_ref().map(DateTimeDue::value),
                "DUE",
            )?;
            dtstart.cmp_exdate(&self.exdate)?;
            dtstart.cmp_rdate(&self.rdate)?;
            dtstart.cmp_exdate_tzid(&self.exdate)?;
            dtstart.cmp_rdate_tzid(&self.rdate)?;
        }
        let alarms = self
            .alarms
            .into_iter()
            .map(AlarmBuilder::build)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Todo {
            dtstamp: self
                .dtstamp
                .ok_or(ComponentError::MissingField("DTSTAMP"))?,
            uid: self.uid.ok_or(ComponentError::MissingField("UID"))?,
            class: self.class,
            completed: self.completed,
            created: self.created,
            description: self.description,
            dtstart: self.dtstart,
            geo: self.geo,
            last_mod: self.last_mod,
            location: self.location,
            organizer: self.organizer,
            percent: self.percent,
            priority: self.priority,
            recur_id: self.recur_id,
            seq: self.seq,
            status: self.status,
            summary: self.summary,
            url: self.url,
            rrule: self.rrule,
            due: self.due,
            duration: self.duration,
            attach: self.attach,
            attendee: self.attendee,
            categories: self.categories,
            comment: self.comment,
            contact: self.contact,
            exdate: self.exdate,
            rstatus: self.rstatus,
            related: self.related,
            resources: self.resources,
            rdate: self.rdate,
            xprop: self.xprop,
            iana: self.iana,
            alarms,
        })
    }
}

impl PropertyIngest for TodoBuilder {
    fn ingest(&mut self, p: Property) -> ParseResult<()> {
        match p {
            Property::DateTimeStamp(v) => {
                set_once(&mut self.dtstamp, v, "DTSTAMP")
            }
            Property::Uid(v) => set_once(&mut self.uid, v, "UID"),
            Property::Classification(v) => {
                set_once(&mut self.class, v, "CLASS")
            }
            Property::Completed(v) => {
                set_once(&mut self.completed, v, "COMPLETED")
            }
            Property::DateTimeCreated(v) => {
                set_once(&mut self.created, v, "CREATED")
            }
            Property::Description(v) => {
                set_once(&mut self.description, v, "DESCRIPTION")
            }
            Property::DateTimeStart(v) => {
                set_once(&mut self.dtstart, v, "DTSTART")
            }
            Property::Geo(v) => set_once(&mut self.geo, v, "GEO"),
            Property::LastModified(v) => {
                set_once(&mut self.last_mod, v, "LAST-MODIFIED")
            }
            Property::Location(v) => {
                set_once(&mut self.location, v, "LOCATION")
            }
            Property::Organizer(v) => {
                set_once(&mut self.organizer, v, "ORGANIZER")
            }
            Property::PercentComplete(v) => {
                set_once(&mut self.percent, v, "PERCENT-COMPLETE")
            }
            Property::Priority(v) => {
                set_once(&mut self.priority, v, "PRIORITY")
            }
            Property::RecurrenceId(v) => {
                set_once(&mut self.recur_id, v, "RECURRENCE-ID")
            }
            Property::Sequence(v) => set_once(&mut self.seq, v, "SEQUENCE"),
            Property::Status(v) => set_once(&mut self.status, v, "STATUS"),
            Property::Summary(v) => set_once(&mut self.summary, v, "SUMMARY"),
            Property::UniformResourceLocator(v) => {
                set_once(&mut self.url, v, "URL")
            }
            Property::RRule(v) => set_once(&mut self.rrule, v, "RRULE"),
            // DUE and DURATION are mutually exclusive within a VTODO, and
            // DURATION requires DTSTART to also be present (RFC 5545
            // §3.6.2) — cross-field rules, checked in `build()`.
            Property::DateTimeDue(v) => set_once(&mut self.due, v, "DUE"),
            Property::Duration(v) => {
                set_once(&mut self.duration, v, "DURATION")
            }
            Property::Attachment(v) => Ok(self.attach.push(v)),
            Property::Attendee(v) => Ok(self.attendee.push(v)),
            Property::Categories(v) => Ok(self.categories.push(v)),
            Property::Comment(v) => Ok(self.comment.push(v)),
            Property::Contact(v) => Ok(self.contact.push(v)),
            Property::ExceptionDateTimes(v) => Ok(self.exdate.push(v)),
            Property::RequestStatus(v) => Ok(self.rstatus.push(v)),
            Property::RelatedTo(v) => Ok(self.related.push(v)),
            Property::Resources(v) => Ok(self.resources.push(v)),
            Property::RecurrenceDateTimes(v) => Ok(self.rdate.push(v)),
            Property::Xprop(v) => Ok(self.xprop.push(v)),
            Property::Iana(v) => Ok(self.iana.push(v)),
            _ => Err(PropertyError::UnexpectedProperty.into()),
        }
    }
}

/// Builder for `VALARM` (RFC 5545 §3.6.6), nested only inside `VEVENT`/
/// `VTODO` (see [`Component::ingest_alarm`]). RFC 5545 actually splits this
/// into three alternative property sets — `audioprop`/`dispprop`/
/// `emailprop` — selected by `ACTION`'s value (AUDIO/DISPLAY/EMAIL), each
/// with its own required/optional/cardinality rules for `DESCRIPTION`,
/// `SUMMARY`, `ATTENDEE`, and `ATTACH`. Ingest only enforces the union of
/// what's structurally possible across all three (hence `Option` here even
/// for fields a given `ACTION` will require, and `Vec` for `ATTACH` even
/// though only `emailprop` allows more than one) — which alternative
/// actually applies, and whether this alarm satisfies it, is a `build()`-
/// time check once `ACTION` is known.
#[derive(Debug, Default)]
struct AlarmBuilder {
    action: Option<Action>,
    trigger: Option<Trigger>,
    // DURATION and REPEAT are optional but MUST appear together (RFC 5545
    // §3.6.6) — a cross-field rule, checked in `build()`.
    duration: Option<Duration>,
    repeat: Option<Repeat>,
    description: Option<Description>,
    summary: Option<Summary>,
    attendee: Vec<Attendee>,
    attach: Vec<Attachment>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
}

impl AlarmBuilder {
    fn new() -> Self {
        Self::default()
    }

    /// Validates the cross-field rules RFC 5545 §3.6.6 places on `VALARM`
    /// and assembles the finished [`Alarm`]. `ACTION` selects which of the
    /// three alternative grammars (`audioprop`/`dispprop`/`emailprop`)
    /// applies — everything beyond "`ACTION` and `TRIGGER` are both
    /// required" is specific to that alternative.
    fn build(self) -> Result<Alarm, ComponentError> {
        let action =
            self.action.ok_or(ComponentError::MissingField("ACTION"))?;
        let trigger = self
            .trigger
            .ok_or(ComponentError::MissingField("TRIGGER"))?;
        if self.duration.is_some() != self.repeat.is_some() {
            return Err(ComponentError::RequiresTogether("DURATION", "REPEAT"));
        }

        match action.kind() {
            ActionEnum::Audio => {
                if self.description.is_some() {
                    return Err(ComponentError::NotAllowed(
                        "DESCRIPTION",
                        "ACTION is AUDIO",
                    ));
                }
                if self.summary.is_some() {
                    return Err(ComponentError::NotAllowed(
                        "SUMMARY",
                        "ACTION is AUDIO",
                    ));
                }
                if !self.attendee.is_empty() {
                    return Err(ComponentError::NotAllowed(
                        "ATTENDEE",
                        "ACTION is AUDIO",
                    ));
                }
                if self.attach.len() > 1 {
                    return Err(ComponentError::NotAllowed(
                        "more than one ATTACH",
                        "ACTION is AUDIO",
                    ));
                }
            }
            ActionEnum::Display => {
                if self.description.is_none() {
                    return Err(ComponentError::MissingField("DESCRIPTION"));
                }
                if self.summary.is_some() {
                    return Err(ComponentError::NotAllowed(
                        "SUMMARY",
                        "ACTION is DISPLAY",
                    ));
                }
                if !self.attendee.is_empty() {
                    return Err(ComponentError::NotAllowed(
                        "ATTENDEE",
                        "ACTION is DISPLAY",
                    ));
                }
                if !self.attach.is_empty() {
                    return Err(ComponentError::NotAllowed(
                        "ATTACH",
                        "ACTION is DISPLAY",
                    ));
                }
            }
            ActionEnum::Email => {
                if self.description.is_none() {
                    return Err(ComponentError::MissingField("DESCRIPTION"));
                }
                if self.summary.is_none() {
                    return Err(ComponentError::MissingField("SUMMARY"));
                }
                if self.attendee.is_empty() {
                    return Err(ComponentError::RequiresAtLeastOne(
                        "ACTION EMAIL",
                        "ATTENDEE",
                    ));
                }
            }
            // IANA/X-name actions aren't defined by RFC 5545 — only the
            // common ACTION/TRIGGER/DURATION+REPEAT rules above apply.
            ActionEnum::Iana(_) | ActionEnum::XName(_) => {}
        }

        Ok(Alarm {
            action,
            trigger,
            duration: self.duration,
            repeat: self.repeat,
            description: self.description,
            summary: self.summary,
            attendee: self.attendee,
            attach: self.attach,
            xprop: self.xprop,
            iana: self.iana,
        })
    }
}

impl PropertyIngest for AlarmBuilder {
    fn ingest(&mut self, p: Property) -> ParseResult<()> {
        match p {
            Property::Action(v) => set_once(&mut self.action, v, "ACTION"),
            Property::Trigger(v) => set_once(&mut self.trigger, v, "TRIGGER"),
            Property::Duration(v) => {
                set_once(&mut self.duration, v, "DURATION")
            }
            Property::Repeat(v) => set_once(&mut self.repeat, v, "REPEAT"),
            Property::Description(v) => {
                set_once(&mut self.description, v, "DESCRIPTION")
            }
            Property::Summary(v) => set_once(&mut self.summary, v, "SUMMARY"),
            Property::Attendee(v) => Ok(self.attendee.push(v)),
            Property::Attachment(v) => Ok(self.attach.push(v)),
            Property::Xprop(v) => Ok(self.xprop.push(v)),
            Property::Iana(v) => Ok(self.iana.push(v)),
            _ => Err(PropertyError::UnexpectedProperty.into()),
        }
    }
}

impl FreeBusyBuilder {
    fn new() -> Self {
        Self::default()
    }

    /// Validates the cross-field rules RFC 5545 §3.6.4 places on
    /// `VFREEBUSY` and assembles the finished [`FreeBusy`].
    fn build(self) -> Result<FreeBusy, ComponentError> {
        if let Some(dtstart) = self.dtstart.as_ref() {
            dtstart.cmp_value_type(
                self.dtend.as_ref().map(DateTimeEnd::value),
                "DTEND",
            )?;
        }
        Ok(FreeBusy {
            dtstamp: self.dtstamp,
            uid: self.uid,
            contact: self.contact,
            dtstart: self.dtstart,
            dtend: self.dtend,
            organizer: self.organizer,
            url: self.url,
            attendee: self.attendee,
            comment: self.comment,
            freebusy: self.freebusy,
            rstatus: self.rstatus,
            xprop: self.xprop,
            iana: self.iana,
        })
    }
}

impl PropertyIngest for FreeBusyBuilder {
    fn ingest(&mut self, p: Property) -> ParseResult<()> {
        match p {
            Property::DateTimeStamp(v) => {
                set_once(&mut self.dtstamp, v, "DTSTAMP")
            }
            Property::Uid(v) => set_once(&mut self.uid, v, "UID"),
            Property::Contact(v) => set_once(&mut self.contact, v, "CONTACT"),
            Property::DateTimeStart(v) => {
                set_once(&mut self.dtstart, v, "DTSTART")
            }
            Property::DateTimeEnd(v) => set_once(&mut self.dtend, v, "DTEND"),
            Property::Organizer(v) => {
                set_once(&mut self.organizer, v, "ORGANIZER")
            }
            Property::UniformResourceLocator(v) => {
                set_once(&mut self.url, v, "URL")
            }
            Property::Attendee(v) => Ok(self.attendee.push(v)),
            Property::Comment(v) => Ok(self.comment.push(v)),
            Property::FreeBusyTime(v) => Ok(self.freebusy.push(v)),
            Property::RequestStatus(v) => Ok(self.rstatus.push(v)),
            Property::Xprop(v) => Ok(self.xprop.push(v)),
            Property::Iana(v) => Ok(self.iana.push(v)),
            _ => Err(PropertyError::UnexpectedProperty.into()),
        }
    }
}
impl JournalBuilder {
    fn new() -> Self {
        Self::default()
    }

    /// Validates the cross-field rules RFC 5545 §3.6.3 places on
    /// `VJOURNAL` and assembles the finished [`Journal`].
    fn build(self) -> Result<Journal, ComponentError> {
        if let Some(dtstart) = self.dtstart.as_ref() {
            dtstart.cmp_until(self.rrule.as_ref())?;
            dtstart.cmp_exdate(&self.exdate)?;
            dtstart.cmp_rdate(&self.rdate)?;
            dtstart.cmp_exdate_tzid(&self.exdate)?;
            dtstart.cmp_rdate_tzid(&self.rdate)?;
        }
        Ok(Journal {
            dtstamp: self
                .dtstamp
                .ok_or(ComponentError::MissingField("DTSTAMP"))?,
            uid: self.uid.ok_or(ComponentError::MissingField("UID"))?,
            class: self.class,
            created: self.created,
            dtstart: self.dtstart,
            last_mod: self.last_mod,
            organizer: self.organizer,
            recurid: self.recurid,
            seq: self.seq,
            status: self.status,
            summary: self.summary,
            url: self.url,
            rrule: self.rrule,
            attach: self.attach,
            attendee: self.attendee,
            categories: self.categories,
            comment: self.comment,
            contact: self.contact,
            description: self.description,
            exdate: self.exdate,
            related: self.related,
            rdate: self.rdate,
            rstatus: self.rstatus,
            xprop: self.xprop,
            iana: self.iana,
        })
    }
}

impl PropertyIngest for JournalBuilder {
    fn ingest(&mut self, p: Property) -> ParseResult<()> {
        match p {
            Property::DateTimeStamp(v) => {
                set_once(&mut self.dtstamp, v, "DTSTAMP")
            }
            Property::Uid(v) => set_once(&mut self.uid, v, "UID"),
            Property::Classification(v) => {
                set_once(&mut self.class, v, "CLASS")
            }
            Property::DateTimeCreated(v) => {
                set_once(&mut self.created, v, "CREATED")
            }
            Property::DateTimeStart(v) => {
                set_once(&mut self.dtstart, v, "DTSTART")
            }
            Property::LastModified(v) => {
                set_once(&mut self.last_mod, v, "LAST-MODIFIED")
            }
            Property::Organizer(v) => {
                set_once(&mut self.organizer, v, "ORGANIZER")
            }
            Property::RecurrenceId(v) => {
                set_once(&mut self.recurid, v, "RECURRENCE-ID")
            }
            Property::Sequence(v) => set_once(&mut self.seq, v, "SEQUENCE"),
            Property::Status(v) => set_once(&mut self.status, v, "STATUS"),
            Property::Summary(v) => set_once(&mut self.summary, v, "SUMMARY"),
            Property::UniformResourceLocator(v) => {
                set_once(&mut self.url, v, "URL")
            }
            Property::RRule(v) => set_once(&mut self.rrule, v, "RRULE"),
            Property::Attachment(v) => Ok(self.attach.push(v)),
            Property::Attendee(v) => Ok(self.attendee.push(v)),
            Property::Categories(v) => Ok(self.categories.push(v)),
            Property::Comment(v) => Ok(self.comment.push(v)),
            Property::Contact(v) => Ok(self.contact.push(v)),
            Property::Description(v) => Ok(self.description.push(v)),
            Property::ExceptionDateTimes(v) => Ok(self.exdate.push(v)),
            Property::RelatedTo(v) => Ok(self.related.push(v)),
            Property::RecurrenceDateTimes(v) => Ok(self.rdate.push(v)),
            Property::RequestStatus(v) => Ok(self.rstatus.push(v)),
            Property::Xprop(v) => Ok(self.xprop.push(v)),
            Property::Iana(v) => Ok(self.iana.push(v)),
            _ => Err(PropertyError::UnexpectedProperty.into()),
        }
    }
}
#[derive(Debug, Default)]
struct FreeBusyBuilder {
    dtstamp: Option<DateTimeStamp>,
    uid: Option<Uid>,
    contact: Option<Contact>,
    dtstart: Option<DateTimeStart>,
    dtend: Option<DateTimeEnd>,
    organizer: Option<Organizer>,
    url: Option<UniformResourceLocator>,
    attendee: Vec<Attendee>,
    comment: Vec<Comment>,
    freebusy: Vec<FreeBusyTime>,
    rstatus: Vec<RequestStatus>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
}

#[derive(Debug, Default)]
struct JournalBuilder {
    dtstamp: Option<DateTimeStamp>,
    uid: Option<Uid>,
    class: Option<Classification>,
    created: Option<DateTimeCreated>,
    dtstart: Option<DateTimeStart>,
    last_mod: Option<LastModified>,
    organizer: Option<Organizer>,
    recurid: Option<RecurrenceId>,
    seq: Option<Sequence>,
    status: Option<Status>,
    summary: Option<Summary>,
    url: Option<UniformResourceLocator>,
    rrule: Option<RRule>,
    attach: Vec<Attachment>,
    attendee: Vec<Attendee>,
    categories: Vec<Categories>,
    comment: Vec<Comment>,
    contact: Vec<Contact>,
    description: Vec<Description>,
    exdate: Vec<ExceptionDateTimes>,
    related: Vec<RelatedTo>,
    rdate: Vec<RecurrenceDateTimes>,
    rstatus: Vec<RequestStatus>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
}

/// Builder for `VTIMEZONE` (RFC 5545 §3.6.5). `standardc`/`daylightc` hold
/// its nested `STANDARD`/`DAYLIGHT` sub-components (see
/// [`crate::ast::parser::Parser::tz_observance`]) — `build()` is still the
/// place that enforces "at least one of either" once every property and
/// sub-component has been seen.
#[derive(Debug, Default)]
struct TimezoneBuilder {
    tzid: Option<TimeZoneIdentifier>,
    last_mod: Option<LastModified>,
    tzurl: Option<TimeZoneUrl>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
    standardc: Vec<TzPropBuilder>,
    daylightc: Vec<TzPropBuilder>,
}

impl TimezoneBuilder {
    fn new() -> Self {
        Self::default()
    }

    /// Validates the cross-field rules RFC 5545 §3.6.5 places on
    /// `VTIMEZONE` and assembles the finished [`Timezone`]. At least one
    /// `STANDARD` or `DAYLIGHT` sub-component is required — that can only
    /// be checked once every nested sub-component has been seen, hence
    /// here rather than in `ingest`.
    fn build(self) -> Result<Timezone, ComponentError> {
        if self.standardc.is_empty() && self.daylightc.is_empty() {
            return Err(ComponentError::RequiresAtLeastOne(
                "VTIMEZONE",
                "STANDARD or DAYLIGHT",
            ));
        }
        let standardc = self
            .standardc
            .into_iter()
            .map(TzPropBuilder::build)
            .collect::<Result<Vec<_>, _>>()?;
        let daylightc = self
            .daylightc
            .into_iter()
            .map(TzPropBuilder::build)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Timezone {
            tzid: self.tzid.ok_or(ComponentError::MissingField("TZID"))?,
            last_mod: self.last_mod,
            tz_url: self.tzurl,
            standardc,
            daylightc,
            xprop: self.xprop,
            iana: self.iana,
        })
    }
}

impl PropertyIngest for TimezoneBuilder {
    fn ingest(&mut self, p: Property) -> ParseResult<()> {
        match p {
            Property::TimeZoneIdentifier(v) => {
                set_once(&mut self.tzid, v, "TZID")
            }
            Property::LastModified(v) => {
                set_once(&mut self.last_mod, v, "LAST-MODIFIED")
            }
            Property::TimeZoneUrl(v) => set_once(&mut self.tzurl, v, "TZURL"),
            Property::Xprop(v) => Ok(self.xprop.push(v)),
            Property::Iana(v) => Ok(self.iana.push(v)),
            _ => Err(PropertyError::UnexpectedProperty.into()),
        }
    }
}

/// Which sub-component produced a given [`TzPropBuilder`] — `STANDARD` and
/// `DAYLIGHT` (RFC 5545 §3.6.5) share an identical `tzprop` property
/// grammar and differ only in which of `VTIMEZONE`'s two observance lists
/// they belong to.
enum TzObservanceKind {
    Standard,
    Daylight,
}

/// Builder for the `tzprop` grammar shared by `STANDARD`/`DAYLIGHT`
/// sub-components (RFC 5545 §3.6.5). Nested only inside `VTIMEZONE`, via
/// [`Component::ingest_tz_observance`].
#[derive(Debug, Default)]
struct TzPropBuilder {
    dtstart: Option<DateTimeStart>,
    tz_offset_to: Option<TimeZoneOffsetTo>,
    tz_offset_from: Option<TimeZoneOffsetFrom>,
    // RRULE is optional but SHOULD NOT occur more than once (RFC 5545
    // §3.6.5) — same "singleton in practice" treatment as elsewhere.
    rrule: Option<RRule>,
    comment: Vec<Comment>,
    rdate: Vec<RecurrenceDateTimes>,
    tzname: Vec<TimeZoneName>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
}

impl TzPropBuilder {
    fn new() -> Self {
        Self::default()
    }

    /// Validates the `tzprop` grammar's required fields (RFC 5545 §3.6.5)
    /// and assembles the finished [`TzProp`].
    fn build(self) -> Result<TzProp, ComponentError> {
        if let Some(dtstart) = self.dtstart.as_ref() {
            dtstart.cmp_until(self.rrule.as_ref())?;
            dtstart.cmp_rdate(&self.rdate)?;
            dtstart.cmp_rdate_tzid(&self.rdate)?;
        }
        Ok(TzProp {
            dtstart: self
                .dtstart
                .ok_or(ComponentError::MissingField("DTSTART"))?,
            tz_offset_to: self
                .tz_offset_to
                .ok_or(ComponentError::MissingField("TZOFFSETTO"))?,
            tz_offset_from: self
                .tz_offset_from
                .ok_or(ComponentError::MissingField("TZOFFSETFROM"))?,
            rrule: self.rrule,
            comment: self.comment,
            rdate: self.rdate,
            tzname: self.tzname,
            xprop: self.xprop,
            iana: self.iana,
        })
    }
}

impl PropertyIngest for TzPropBuilder {
    fn ingest(&mut self, p: Property) -> ParseResult<()> {
        match p {
            Property::DateTimeStart(v) => {
                set_once(&mut self.dtstart, v, "DTSTART")
            }
            Property::TimeZoneOffsetTo(v) => {
                set_once(&mut self.tz_offset_to, v, "TZOFFSETTO")
            }
            Property::TimeZoneOffsetFrom(v) => {
                set_once(&mut self.tz_offset_from, v, "TZOFFSETFROM")
            }
            Property::RRule(v) => set_once(&mut self.rrule, v, "RRULE"),
            Property::Comment(v) => Ok(self.comment.push(v)),
            Property::RecurrenceDateTimes(v) => Ok(self.rdate.push(v)),
            Property::TimeZoneName(v) => Ok(self.tzname.push(v)),
            Property::Xprop(v) => Ok(self.xprop.push(v)),
            Property::Iana(v) => Ok(self.iana.push(v)),
            _ => Err(PropertyError::UnexpectedProperty.into()),
        }
    }
}

#[cfg(test)]
mod build_tests {
    use super::*;

    fn prop(name: &[u8], remainder: &[u8]) -> Property {
        Property::parse(name, remainder).unwrap()
    }

    fn minimal_event() -> EventBuilder {
        let mut b = EventBuilder::new();
        b.ingest(prop(b"DTSTAMP", b":19970901T130000Z")).unwrap();
        b.ingest(prop(b"UID", b":123@example.com")).unwrap();
        b.ingest(prop(b"DTSTART", b":19970903T163000Z")).unwrap();
        b
    }

    #[test]
    fn event_requires_dtstamp() {
        let mut b = minimal_event();
        b.dtstamp = None;
        assert!(matches!(
            b.build(true),
            Err(ComponentError::MissingField("DTSTAMP"))
        ));
    }

    #[test]
    fn event_requires_uid() {
        let mut b = minimal_event();
        b.uid = None;
        assert!(matches!(
            b.build(true),
            Err(ComponentError::MissingField("UID"))
        ));
    }

    #[test]
    fn event_dtstart_required_unless_method_present() {
        let mut b = minimal_event();
        b.dtstart = None;
        assert!(matches!(
            b.build(false),
            Err(ComponentError::MissingField("DTSTART"))
        ));
        assert!(minimal_event().build(true).is_ok());
    }

    #[test]
    fn event_dtend_and_duration_are_mutually_exclusive() {
        let mut b = minimal_event();
        b.ingest(prop(b"DTEND", b":19970903T190000Z")).unwrap();
        b.ingest(prop(b"DURATION", b":PT1H")).unwrap();
        assert!(matches!(
            b.build(true),
            Err(ComponentError::MutuallyExclusive("DTEND", "DURATION"))
        ));
    }

    #[test]
    fn event_builds_with_only_required_fields() {
        let event = minimal_event().build(true).unwrap();
        assert!(event.dtstart.is_some());
        assert!(event.alarms.is_empty());
    }

    #[test]
    fn event_rrule_until_must_match_dtstart_value_type() {
        // minimal_event's DTSTART is DATE-TIME; UNTIL here is DATE.
        let mut b = minimal_event();
        b.ingest(prop(b"RRULE", b":FREQ=DAILY;UNTIL=19971224"))
            .unwrap();
        assert!(matches!(
            b.build(true),
            Err(ComponentError::MismatchedValueType(
                "RRULE's UNTIL",
                "DTSTART"
            ))
        ));
    }

    #[test]
    fn event_rrule_until_matching_dtstart_value_type_is_ok() {
        let mut b = minimal_event();
        b.ingest(prop(b"RRULE", b":FREQ=DAILY;UNTIL=19971224T000000Z"))
            .unwrap();
        assert!(b.build(true).is_ok());
    }

    #[test]
    fn event_dtend_must_match_dtstart_value_type() {
        // minimal_event's DTSTART is DATE-TIME; DTEND here is DATE.
        let mut b = minimal_event();
        b.ingest(prop(b"DTEND", b";VALUE=DATE:19971009")).unwrap();
        assert!(matches!(
            b.build(true),
            Err(ComponentError::MismatchedValueType("DTEND", "DTSTART"))
        ));
    }

    #[test]
    fn event_dtend_matching_dtstart_value_type_is_ok() {
        let mut b = minimal_event();
        b.ingest(prop(b"DTEND", b":19970903T190000Z")).unwrap();
        assert!(b.build(true).is_ok());
    }

    #[test]
    fn event_exdate_must_match_dtstart_value_type() {
        // minimal_event's DTSTART is DATE-TIME; EXDATE here is DATE.
        let mut b = minimal_event();
        b.ingest(prop(b"EXDATE", b";VALUE=DATE:19971009")).unwrap();
        assert!(matches!(
            b.build(true),
            Err(ComponentError::MismatchedValueType("EXDATE", "DTSTART"))
        ));
    }

    #[test]
    fn event_exdate_matching_dtstart_value_type_is_ok() {
        let mut b = minimal_event();
        b.ingest(prop(b"EXDATE", b":19970904T163000Z,19970905T163000Z"))
            .unwrap();
        assert!(b.build(true).is_ok());
    }

    #[test]
    fn event_rdate_must_match_dtstart_value_type() {
        // minimal_event's DTSTART is DATE-TIME; RDATE here is DATE.
        let mut b = minimal_event();
        b.ingest(prop(b"RDATE", b";VALUE=DATE:19971009")).unwrap();
        assert!(matches!(
            b.build(true),
            Err(ComponentError::MismatchedValueType("RDATE", "DTSTART"))
        ));
    }

    #[test]
    fn event_rdate_period_is_allowed_regardless_of_dtstart_value_type() {
        // A PERIOD-valued RDATE is always allowed (RFC 5545 §3.8.5.2), even
        // though minimal_event's DTSTART is DATE-TIME rather than DATE.
        let mut b = minimal_event();
        b.ingest(prop(b"RDATE", b":19970903T163000Z/PT2H")).unwrap();
        assert!(b.build(true).is_ok());
    }

    fn minimal_event_with_zoned_dtstart() -> EventBuilder {
        let mut b = minimal_event();
        b.dtstart = Some(
            DateTimeStart::try_from(
                b";TZID=America/New_York:20130907T120000".as_slice(),
            )
            .unwrap(),
        );
        b
    }

    #[test]
    fn event_exdate_tzid_matching_dtstart_is_ok() {
        // tests/fixtures/collective-icalendar/events/
        // issue_112_missing_tzinfo_on_exdate.ics
        let mut b = minimal_event_with_zoned_dtstart();
        b.ingest(prop(b"EXDATE", b";TZID=America/New_York:20131012T120000"))
            .unwrap();
        b.ingest(prop(b"EXDATE", b";TZID=America/New_York:20131011T120000"))
            .unwrap();
        assert!(b.build(true).is_ok());
    }

    #[test]
    fn event_exdate_tzid_must_match_dtstart_tzid() {
        // Both DATE-TIME (value type matches), but different zones — a
        // mismatch the value-type check alone can't catch.
        let mut b = minimal_event_with_zoned_dtstart();
        b.ingest(prop(b"EXDATE", b";TZID=Europe/London:20131012T120000"))
            .unwrap();
        assert!(matches!(
            b.build(true),
            Err(ComponentError::MismatchedTzid("EXDATE", "DTSTART"))
        ));
    }

    #[test]
    fn event_exdate_with_no_tzid_must_match_a_zoned_dtstart() {
        // DTSTART carries a TZID but this EXDATE carries none — a
        // UTC/floating value doesn't name the same instant/wall-clock time
        // as the zoned DTSTART.
        let mut b = minimal_event_with_zoned_dtstart();
        b.ingest(prop(b"EXDATE", b":20131012T120000Z")).unwrap();
        assert!(matches!(
            b.build(true),
            Err(ComponentError::MismatchedTzid("EXDATE", "DTSTART"))
        ));
    }

    #[test]
    fn event_rdate_tzid_matching_dtstart_is_ok() {
        let mut b = minimal_event_with_zoned_dtstart();
        b.ingest(prop(b"RDATE", b";TZID=America/New_York:20130914T120000"))
            .unwrap();
        assert!(b.build(true).is_ok());
    }

    #[test]
    fn event_rdate_tzid_must_match_dtstart_tzid() {
        let mut b = minimal_event_with_zoned_dtstart();
        b.ingest(prop(b"RDATE", b";TZID=Europe/London:20130914T120000"))
            .unwrap();
        assert!(matches!(
            b.build(true),
            Err(ComponentError::MismatchedTzid("RDATE", "DTSTART"))
        ));
    }

    fn minimal_todo() -> TodoBuilder {
        let mut b = TodoBuilder::new();
        b.ingest(prop(b"DTSTAMP", b":19970901T130000Z")).unwrap();
        b.ingest(prop(b"UID", b":123@example.com")).unwrap();
        b
    }

    #[test]
    fn todo_due_and_duration_are_mutually_exclusive() {
        let mut b = minimal_todo();
        b.ingest(prop(b"DTSTART", b":19970901T130000Z")).unwrap();
        b.ingest(prop(b"DUE", b":19970902T130000Z")).unwrap();
        b.ingest(prop(b"DURATION", b":PT1H")).unwrap();
        assert!(matches!(
            b.build(),
            Err(ComponentError::MutuallyExclusive("DUE", "DURATION"))
        ));
    }

    #[test]
    fn todo_duration_requires_dtstart() {
        let mut b = minimal_todo();
        b.ingest(prop(b"DURATION", b":PT1H")).unwrap();
        assert!(matches!(
            b.build(),
            Err(ComponentError::Requires("DURATION", "DTSTART"))
        ));
    }

    #[test]
    fn todo_builds_with_only_required_fields() {
        assert!(minimal_todo().build().is_ok());
    }

    #[test]
    fn todo_due_must_match_dtstart_value_type() {
        let mut b = minimal_todo();
        b.ingest(prop(b"DTSTART", b";VALUE=DATE:20070430")).unwrap();
        b.ingest(prop(b"DUE", b":20070501T000000Z")).unwrap();
        assert!(matches!(
            b.build(),
            Err(ComponentError::MismatchedValueType("DUE", "DTSTART"))
        ));
    }

    #[test]
    fn todo_due_matching_dtstart_value_type_is_ok() {
        let mut b = minimal_todo();
        b.ingest(prop(b"DTSTART", b";VALUE=DATE:20070430")).unwrap();
        b.ingest(prop(b"DUE", b";VALUE=DATE:20070501")).unwrap();
        assert!(b.build().is_ok());
    }

    #[test]
    fn todo_exdate_and_rdate_must_match_dtstart_value_type() {
        let mut b = minimal_todo();
        b.ingest(prop(b"DTSTART", b";VALUE=DATE:20070430")).unwrap();
        b.ingest(prop(b"EXDATE", b":20070501T000000Z")).unwrap();
        assert!(matches!(
            b.build(),
            Err(ComponentError::MismatchedValueType("EXDATE", "DTSTART"))
        ));
    }

    fn minimal_alarm(action: &[u8]) -> AlarmBuilder {
        let mut b = AlarmBuilder::new();
        b.ingest(prop(b"ACTION", action)).unwrap();
        b.ingest(prop(b"TRIGGER", b":-PT15M")).unwrap();
        b
    }

    #[test]
    fn alarm_requires_action_and_trigger() {
        assert!(matches!(
            AlarmBuilder::new().build(),
            Err(ComponentError::MissingField("ACTION"))
        ));

        let mut with_action = AlarmBuilder::new();
        with_action.ingest(prop(b"ACTION", b":AUDIO")).unwrap();
        assert!(matches!(
            with_action.build(),
            Err(ComponentError::MissingField("TRIGGER"))
        ));
    }

    #[test]
    fn alarm_duration_and_repeat_must_occur_together() {
        let mut b = minimal_alarm(b":AUDIO");
        b.ingest(prop(b"DURATION", b":PT15M")).unwrap();
        assert!(matches!(
            b.build(),
            Err(ComponentError::RequiresTogether("DURATION", "REPEAT"))
        ));
    }

    #[test]
    fn alarm_audio_rejects_description() {
        let mut b = minimal_alarm(b":AUDIO");
        b.ingest(prop(b"DESCRIPTION", b":Beep")).unwrap();
        assert!(matches!(
            b.build(),
            Err(ComponentError::NotAllowed("DESCRIPTION", _))
        ));
    }

    #[test]
    fn alarm_display_requires_description() {
        let b = minimal_alarm(b":DISPLAY");
        assert!(matches!(
            b.build(),
            Err(ComponentError::MissingField("DESCRIPTION"))
        ));
    }

    #[test]
    fn alarm_email_requires_description_summary_and_attendee() {
        assert!(matches!(
            minimal_alarm(b":EMAIL").build(),
            Err(ComponentError::MissingField("DESCRIPTION"))
        ));

        let mut with_description = minimal_alarm(b":EMAIL");
        with_description
            .ingest(prop(b"DESCRIPTION", b":Reminder"))
            .unwrap();
        assert!(matches!(
            with_description.build(),
            Err(ComponentError::MissingField("SUMMARY"))
        ));

        let mut with_summary = minimal_alarm(b":EMAIL");
        with_summary
            .ingest(prop(b"DESCRIPTION", b":Reminder"))
            .unwrap();
        with_summary.ingest(prop(b"SUMMARY", b":Reminder")).unwrap();
        assert!(matches!(
            with_summary.build(),
            Err(ComponentError::RequiresAtLeastOne(
                "ACTION EMAIL",
                "ATTENDEE"
            ))
        ));

        let mut complete = minimal_alarm(b":EMAIL");
        complete.ingest(prop(b"DESCRIPTION", b":Reminder")).unwrap();
        complete.ingest(prop(b"SUMMARY", b":Reminder")).unwrap();
        complete
            .ingest(prop(b"ATTENDEE", b":mailto:jane@example.com"))
            .unwrap();
        assert!(complete.build().is_ok());
    }

    #[test]
    fn alarm_audio_builds_with_only_required_fields() {
        assert!(minimal_alarm(b":AUDIO").build().is_ok());
    }

    fn minimal_tz_prop() -> TzPropBuilder {
        let mut b = TzPropBuilder::new();
        b.ingest(prop(b"DTSTART", b":19710101T020000")).unwrap();
        b.ingest(prop(b"TZOFFSETFROM", b":-0400")).unwrap();
        b.ingest(prop(b"TZOFFSETTO", b":-0500")).unwrap();
        b
    }

    #[test]
    fn timezone_requires_at_least_one_observance() {
        let mut b = TimezoneBuilder::new();
        b.ingest(prop(b"TZID", b":America/New_York")).unwrap();
        assert!(matches!(
            b.build(),
            Err(ComponentError::RequiresAtLeastOne(
                "VTIMEZONE",
                "STANDARD or DAYLIGHT"
            ))
        ));
    }

    #[test]
    fn timezone_builds_with_one_standard_observance() {
        let mut b = TimezoneBuilder::new();
        b.ingest(prop(b"TZID", b":America/New_York")).unwrap();
        b.standardc.push(minimal_tz_prop());
        assert!(b.build().is_ok());
    }

    #[test]
    fn tz_prop_requires_dtstart_and_offsets() {
        assert!(matches!(
            TzPropBuilder::new().build(),
            Err(ComponentError::MissingField("DTSTART"))
        ));
    }

    #[test]
    fn tz_prop_rrule_until_must_match_dtstart_value_type() {
        // minimal_tz_prop's DTSTART is DATE-TIME; UNTIL here is DATE.
        let mut b = minimal_tz_prop();
        b.ingest(prop(b"RRULE", b":FREQ=YEARLY;UNTIL=20070311"))
            .unwrap();
        assert!(matches!(
            b.build(),
            Err(ComponentError::MismatchedValueType(
                "RRULE's UNTIL",
                "DTSTART"
            ))
        ));
    }

    #[test]
    fn tz_prop_rdate_must_match_dtstart_value_type() {
        // minimal_tz_prop's DTSTART is DATE-TIME; RDATE here is DATE.
        let mut b = minimal_tz_prop();
        b.ingest(prop(b"RDATE", b";VALUE=DATE:20070311")).unwrap();
        assert!(matches!(
            b.build(),
            Err(ComponentError::MismatchedValueType("RDATE", "DTSTART"))
        ));
    }

    fn minimal_journal() -> JournalBuilder {
        let mut b = JournalBuilder::new();
        b.ingest(prop(b"DTSTAMP", b":19970901T130000Z")).unwrap();
        b.ingest(prop(b"UID", b":123@example.com")).unwrap();
        b.ingest(prop(b"DTSTART", b";VALUE=DATE:19970317")).unwrap();
        b
    }

    #[test]
    fn journal_exdate_must_match_dtstart_value_type() {
        // minimal_journal's DTSTART is DATE; EXDATE here is DATE-TIME.
        let mut b = minimal_journal();
        b.ingest(prop(b"EXDATE", b":19970318T000000Z")).unwrap();
        assert!(matches!(
            b.build(),
            Err(ComponentError::MismatchedValueType("EXDATE", "DTSTART"))
        ));
    }

    #[test]
    fn journal_rdate_matching_dtstart_value_type_is_ok() {
        let mut b = minimal_journal();
        b.ingest(prop(b"RDATE", b";VALUE=DATE:19970318")).unwrap();
        assert!(b.build().is_ok());
    }

    fn minimal_freebusy() -> FreeBusyBuilder {
        let mut b = FreeBusyBuilder::new();
        b.ingest(prop(b"DTSTAMP", b":19970901T083000Z")).unwrap();
        b.ingest(prop(b"UID", b":123@example.com")).unwrap();
        b.ingest(prop(b"DTSTART", b":19971015T050000Z")).unwrap();
        b
    }

    #[test]
    fn freebusy_dtend_must_match_dtstart_value_type() {
        // minimal_freebusy's DTSTART is DATE-TIME; DTEND here is DATE.
        let mut b = minimal_freebusy();
        b.ingest(prop(b"DTEND", b";VALUE=DATE:19971016")).unwrap();
        assert!(matches!(
            b.build(),
            Err(ComponentError::MismatchedValueType("DTEND", "DTSTART"))
        ));
    }

    #[test]
    fn freebusy_dtend_matching_dtstart_value_type_is_ok() {
        let mut b = minimal_freebusy();
        b.ingest(prop(b"DTEND", b":19971016T050000Z")).unwrap();
        assert!(b.build().is_ok());
    }

    #[test]
    fn calendar_requires_prodid_version_and_a_component() {
        assert!(matches!(
            CalendarBuilder::new().build(),
            Err(ComponentError::MissingField("PRODID"))
        ));

        let mut with_prodid = CalendarBuilder::new();
        with_prodid.prodid = Some(
            ProductIdentifier::try_from(b":-//example//EN".as_slice()).unwrap(),
        );
        assert!(matches!(
            with_prodid.build(),
            Err(ComponentError::MissingField("VERSION"))
        ));

        let mut with_version = CalendarBuilder::new();
        with_version.prodid = Some(
            ProductIdentifier::try_from(b":-//example//EN".as_slice()).unwrap(),
        );
        with_version.version =
            Some(Version::try_from(b":2.0".as_slice()).unwrap());
        assert!(matches!(
            with_version.build(),
            Err(ComponentError::RequiresAtLeastOne(
                "VCALENDAR",
                "calendar component"
            ))
        ));

        let mut complete = CalendarBuilder::new();
        complete.prodid = Some(
            ProductIdentifier::try_from(b":-//example//EN".as_slice()).unwrap(),
        );
        complete.version = Some(Version::try_from(b":2.0".as_slice()).unwrap());
        complete.components.push(minimal_event().into());
        assert!(complete.build().is_ok());
    }

    fn minimal_calendar(components: Vec<Component>) -> CalendarBuilder {
        let mut b = CalendarBuilder::new();
        b.prodid = Some(
            ProductIdentifier::try_from(b":-//example//EN".as_slice()).unwrap(),
        );
        b.version = Some(Version::try_from(b":2.0".as_slice()).unwrap());
        b.components = components;
        b
    }

    fn minimal_timezone(tzid: &[u8]) -> TimezoneBuilder {
        let mut b = TimezoneBuilder::new();
        b.ingest(prop(b"TZID", tzid)).unwrap();
        b.standardc.push(minimal_tz_prop());
        b
    }

    #[test]
    fn calendar_accepts_a_tzid_reference_with_no_matching_vtimezone() {
        // RFC 5545 §3.6.5 says a VTIMEZONE "MUST" back every referenced
        // TZID, but a TZID that reached this point as a `TimeZoneIdentifier`
        // has, by construction, already been resolved against chrono_tz's
        // own IANA database — a real, independent source of offset rules.
        // Requiring a redundant local VTIMEZONE on top of that would reject
        // the majority of real-world calendars (issue #14).
        let mut event = minimal_event();
        event.dtstart = Some(
            DateTimeStart::try_from(
                b";TZID=America/New_York:19970903T163000".as_slice(),
            )
            .unwrap(),
        );
        let cal = minimal_calendar(vec![event.into()]);
        assert!(cal.build().is_ok());
    }

    #[test]
    fn calendar_accepts_a_tzid_reference_backed_by_a_vtimezone() {
        let mut event = minimal_event();
        event.dtstart = Some(
            DateTimeStart::try_from(
                b";TZID=America/New_York:19970903T163000".as_slice(),
            )
            .unwrap(),
        );
        let cal = minimal_calendar(vec![
            event.into(),
            minimal_timezone(b":America/New_York").into(),
        ]);
        assert!(cal.build().is_ok());
    }

    #[test]
    fn calendar_rejects_the_same_tzid_defined_by_two_vtimezones() {
        let cal = minimal_calendar(vec![
            minimal_event().into(),
            minimal_timezone(b":America/New_York").into(),
            minimal_timezone(b":America/New_York").into(),
        ]);
        assert!(matches!(
            cal.build(),
            Err(ComponentError::DuplicateTimeZone(tz))
                if tz == "America/New_York"
        ));
    }

    #[test]
    fn dtstart_rejects_a_non_iana_tzid() {
        // tests/fixtures/collective-icalendar/calendars/issue_218_bad_tzid.ics
        // — `DTSTART;TZID=UTC+11:...`. `UTC+11` isn't an IANA zone name, so
        // this must surface as a clear parse error rather than being
        // silently accepted (see the `TimeZoneIdentifier` doc comment in
        // `params.rs`).
        assert!(matches!(
            DateTimeStart::try_from(b";TZID=UTC+11:20170228T230000".as_slice(),),
            Err(ParseError::Parameters(ParameterError::Param(
                crate::params::ParamError::Malformed { .. }
            )))
        ));
    }

    #[test]
    fn calendar_rejects_two_components_sharing_uid_and_no_recurrence_id() {
        let cal = minimal_calendar(vec![
            minimal_event().into(),
            minimal_event().into(),
        ]);
        assert!(matches!(
            cal.build(),
            Err(ComponentError::DuplicateUid(uid))
                if uid == "123@example.com"
        ));
    }

    #[test]
    fn calendar_allows_shared_uid_with_distinct_recurrence_ids() {
        let mut overridden = minimal_event();
        overridden
            .ingest(prop(b"RECURRENCE-ID", b":19970904T163000Z"))
            .unwrap();
        let cal =
            minimal_calendar(vec![minimal_event().into(), overridden.into()]);
        assert!(cal.build().is_ok());
    }
}
