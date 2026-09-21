#[cfg(feature = "rfc-7953")]
use crate::components::availability::Availability;
use crate::{
    ComponentError,
    ast::{
        Lexer, LexerError,
        parser::{ParseError, Parser},
        validate_calendar,
    },
    components::{
        event::Event, free_busy::FreeBusy, journal::Journal,
        timezone::Timezone, todo::Todo, unknown::UnknownComponent, write_lines,
    },
    properties::{
        CalendarScale, Categories, Color, Description, Iana, Image,
        LastModified, Method, Name, ProductIdentifier, RefreshInterval, Source,
        Uid, UniformResourceLocator, Version, Xprop,
    },
};

/// The Calendaring and Scheduling Core Object is a collection of
/// calendaring and scheduling information.  Typically, this information
/// will consist of an iCalendar stream with a single iCalendar object.
/// However, multiple iCalendar objects can be sequentially grouped
/// together in an iCalendar stream.  The first line and last line of the
/// iCalendar object MUST contain a pair of iCalendar object delimiter
/// strings.
///
/// The body of the iCalendar object consists of a sequence of calendar
/// properties and one or more calendar components.  The calendar
/// properties are attributes that apply to the calendar object as a
/// whole.  The calendar components are collections of properties that
/// express a particular calendar semantic.  For example, the calendar
/// component can specify an event, a to-do, a journal entry, time zone
/// information, free/busy time information, or an alarm.
///
/// An iCalendar object MUST include the "PRODID" and "VERSION" calendar
/// properties.  In addition, it MUST include at least one calendar
/// component.  Special forms of iCalendar objects are possible to
/// publish just busy time (i.e., only a "VFREEBUSY" calendar component)
/// or time zone (i.e., only a "VTIMEZONE" calendar component)
/// information.  In addition, a complex iCalendar object that is used to
/// capture a complete snapshot of the contents of a calendar is possible
/// (e.g., composite of many different calendar components).  More
/// commonly, an iCalendar object will consist of just a single "VEVENT",
/// "VTODO", or "VJOURNAL" calendar component.  Applications MUST ignore
/// x-comp and iana-comp values they don't recognize.  Applications that
/// support importing iCalendar objects SHOULD support all of the
/// component types defined in this document, and SHOULD NOT silently
/// drop any components as that can lead to user data loss.
#[derive(Debug)]
pub struct Calendar {
    pub(crate) prodid: ProductIdentifier,
    pub(crate) version: Version,
    pub(crate) calscale: Option<CalendarScale>,
    pub(crate) method: Option<Method>,
    // RFC 7986 §5 new/extended `VCALENDAR`-level properties. RFC 7986
    // updates RFC 5545 itself (unlike RFC 7953/9074's distinct new
    // components/extensions), so this crate treats it as core: always
    // compiled in, no feature flag.
    pub(crate) uid: Option<Uid>,
    pub(crate) last_mod: Option<LastModified>,
    pub(crate) url: Option<UniformResourceLocator>,
    pub(crate) refresh_interval: Option<RefreshInterval>,
    pub(crate) source: Option<Source>,
    pub(crate) color: Option<Color>,
    pub(crate) name: Vec<Name>,
    pub(crate) description: Vec<Description>,
    pub(crate) categories: Vec<Categories>,
    pub(crate) image: Vec<Image>,
    pub(crate) xprop: Vec<Xprop>,
    pub(crate) iana: Vec<Iana>,
    pub(crate) components: Vec<Component>,
}

/// The calendar component carried by a built [`Calendar`] — the built
/// counterpart of [`crate::ast::Component`], which wraps the in-progress
/// builders instead. One variant per component type RFC 5545 §3.6 allows
/// directly under `VCALENDAR`; see [`crate::ast`]'s module docs for why a
/// sub-component (`VALARM`, `STANDARD`, `DAYLIGHT`) never gets a variant
/// here.
#[derive(Debug)]
pub enum Component {
    /// A scheduled event (`VEVENT`).
    Event(Event),
    /// A to-do task (`VTODO`).
    Todo(Todo),
    /// A journal entry (`VJOURNAL`).
    Journal(Journal),
    /// Free/busy time information (`VFREEBUSY`).
    FreeBusy(FreeBusy),
    /// Time zone definition (`VTIMEZONE`).
    Timezone(Timezone),
    /// Availability information (`VAVAILABILITY`, RFC 7953 §3.1).
    #[cfg(feature = "rfc-7953")]
    Availability(Availability),
    /// An unrecognized `iana-comp`/`x-comp` (RFC 5545 §3.6), preserved
    /// verbatim rather than dropped. See [`UnknownComponent`].
    Unknown(UnknownComponent),
}

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Event(c) => write!(f, "{c}"),
            Self::Todo(c) => write!(f, "{c}"),
            Self::Journal(c) => write!(f, "{c}"),
            Self::FreeBusy(c) => write!(f, "{c}"),
            Self::Timezone(c) => write!(f, "{c}"),
            #[cfg(feature = "rfc-7953")]
            Self::Availability(c) => write!(f, "{c}"),
            Self::Unknown(c) => write!(f, "{c}"),
        }
    }
}

impl std::fmt::Display for Calendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:VCALENDAR\r\n")?;
        write!(f, "{}\r\n", self.prodid)?;
        write!(f, "{}\r\n", self.version)?;
        if let Some(v) = &self.calscale {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.method {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.uid {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.last_mod {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.url {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.refresh_interval {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.source {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.color {
            write!(f, "{v}\r\n")?;
        }
        write_lines(f, &self.name)?;
        write_lines(f, &self.description)?;
        write_lines(f, &self.categories)?;
        write_lines(f, &self.image)?;
        write_lines(f, &self.xprop)?;
        write_lines(f, &self.iana)?;
        for c in &self.components {
            write!(f, "{c}")?;
        }
        write!(f, "END:VCALENDAR\r\n")
    }
}

impl Calendar {
    /// The calendar components (`VEVENT`, `VTODO`, `VJOURNAL`, `VFREEBUSY`,
    /// `VTIMEZONE`) carried by this `VCALENDAR` object.
    pub fn components(&self) -> &[Component] {
        &self.components
    }

    /// Parses one `BEGIN:VCALENDAR ... END:VCALENDAR` iCalendar object (RFC
    /// 5545 §3.4/§3.6) from raw bytes.
    ///
    /// This is the public entry point into the crate's parser: it tokenizes
    /// `src` and builds a [`Calendar`] from it, running every cross-field
    /// validation deferred to `build()` (see the [`crate::ast`] module docs).
    /// A stream containing more than one `icalobject` back to back isn't
    /// supported by this function — it parses exactly one `VCALENDAR` and
    /// errors if anything follows. Use [`Calendar::parse_stream`] for a
    /// stream of several.
    pub fn parse(src: &[u8]) -> Result<Self, CalendarParseError> {
        let tokens = Lexer::new(src).scan()?;
        Parser::new(tokens).calendar().map_err(Into::into)
    }

    /// Parses an iCalendar stream (RFC 5545 §3.4) of one or more back-to-back
    /// `BEGIN:VCALENDAR ... END:VCALENDAR` objects (`icalstream =
    /// 1*icalobject`), returning one [`Calendar`] per object, in order.
    ///
    /// RFC 5545 explicitly allows several `icalobject`s to be grouped
    /// sequentially in one stream — e.g. a full calendar export composed of
    /// several independently-produced `VCALENDAR`s concatenated together.
    /// [`Calendar::parse`] only ever accepts exactly one object and errors
    /// on anything trailing it; this is the entry point for the general
    /// case.
    ///
    /// Example:
    ///
    /// > BEGIN:VCALENDAR
    /// > ...
    /// > END:VCALENDAR
    /// > BEGIN:VCALENDAR
    /// > ...
    /// > END:VCALENDAR
    ///
    /// [Section 3.4](https://datatracker.ietf.org/doc/html/rfc5545#section-3.4)
    pub fn parse_stream(src: &[u8]) -> Result<Vec<Self>, CalendarParseError> {
        let tokens = Lexer::new(src).scan()?;
        let mut parser = Parser::new(tokens);
        let mut calendars = Vec::new();
        while !parser.is_at_end()? {
            calendars.push(parser.calendar()?);
        }
        Ok(calendars)
    }
}

/// Builder for [`Calendar`], for constructing a `VCALENDAR` from code rather
/// than parsing one. Mirrors [`crate::components::event::EventBuilder`]: the
/// two properties RFC 5545 §3.7 requires unconditionally (`PRODID`,
/// `VERSION`) go in [`CalendarBuilder::new`], everything else is an optional
/// chained setter, and [`CalendarBuilder::build`] (or, equivalently,
/// `Calendar::try_from`) runs every cross-field rule that needs the whole
/// object — at least one component, `DTSTART` on every `VEVENT` when there's
/// no `METHOD`, unique `VTIMEZONE`s, no two components claiming the same
/// `UID`/`RECURRENCE-ID`, and RFC 5546's per-`METHOD` tables when the
/// `rfc-5546` feature is on. These are the same checks the parser applies,
/// so a `Calendar` is held to the same rules however it was assembled.
///
/// Components are added already built ([`Event`], [`Todo`], ...), each
/// having passed its own builder's validation.
///
/// Example:
///
/// > BEGIN:VCALENDAR
/// > PRODID:-//example//EN
/// > VERSION:2.0
/// > BEGIN:VEVENT
/// > ...
/// > END:VEVENT
/// > END:VCALENDAR
///
/// [Section 3.4](https://datatracker.ietf.org/doc/html/rfc5545#section-3.4)
#[derive(Debug)]
pub struct CalendarBuilder {
    prodid: ProductIdentifier,
    version: Version,
    calscale: Option<CalendarScale>,
    method: Option<Method>,
    uid: Option<Uid>,
    last_mod: Option<LastModified>,
    url: Option<UniformResourceLocator>,
    refresh_interval: Option<RefreshInterval>,
    source: Option<Source>,
    color: Option<Color>,
    name: Vec<Name>,
    description: Vec<Description>,
    categories: Vec<Categories>,
    image: Vec<Image>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
    components: Vec<Component>,
}

impl CalendarBuilder {
    /// Starts building a `VCALENDAR` from its two unconditionally REQUIRED
    /// properties, `PRODID` and `VERSION` (RFC 5545 §3.7.3, §3.7.4).
    pub fn new(prodid: ProductIdentifier, version: Version) -> Self {
        Self {
            prodid,
            version,
            calscale: None,
            method: None,
            uid: None,
            last_mod: None,
            url: None,
            refresh_interval: None,
            source: None,
            color: None,
            name: Vec::new(),
            description: Vec::new(),
            categories: Vec::new(),
            image: Vec::new(),
            xprop: Vec::new(),
            iana: Vec::new(),
            components: Vec::new(),
        }
    }

    /// Sets `CALSCALE`.
    pub fn calscale(mut self, v: CalendarScale) -> Self {
        self.calscale = Some(v);
        self
    }

    /// Sets `METHOD`. When set, a `VEVENT` no longer needs a `DTSTART`
    /// (RFC 5545 §3.6.1).
    pub fn method(mut self, v: Method) -> Self {
        self.method = Some(v);
        self
    }

    /// Sets `UID` (RFC 7986 §5.3).
    pub fn uid(mut self, v: Uid) -> Self {
        self.uid = Some(v);
        self
    }

    /// Sets `LAST-MODIFIED`.
    pub fn last_mod(mut self, v: LastModified) -> Self {
        self.last_mod = Some(v);
        self
    }

    /// Sets `URL`.
    pub fn url(mut self, v: UniformResourceLocator) -> Self {
        self.url = Some(v);
        self
    }

    /// Sets `REFRESH-INTERVAL`.
    pub fn refresh_interval(mut self, v: RefreshInterval) -> Self {
        self.refresh_interval = Some(v);
        self
    }

    /// Sets `SOURCE`.
    pub fn source(mut self, v: Source) -> Self {
        self.source = Some(v);
        self
    }

    /// Sets `COLOR`.
    pub fn color(mut self, v: Color) -> Self {
        self.color = Some(v);
        self
    }

    /// Adds a `NAME`; may be called more than once.
    pub fn name(mut self, v: Name) -> Self {
        self.name.push(v);
        self
    }

    /// Adds a `DESCRIPTION`; may be called more than once.
    pub fn description(mut self, v: Description) -> Self {
        self.description.push(v);
        self
    }

    /// Adds a `CATEGORIES`; may be called more than once.
    pub fn categories(mut self, v: Categories) -> Self {
        self.categories.push(v);
        self
    }

    /// Adds an `IMAGE`; may be called more than once.
    pub fn image(mut self, v: Image) -> Self {
        self.image.push(v);
        self
    }

    /// Adds an `X-` property; may be called more than once.
    pub fn xprop(mut self, v: Xprop) -> Self {
        self.xprop.push(v);
        self
    }

    /// Adds an IANA-registered property this crate doesn't otherwise
    /// model; may be called more than once.
    pub fn iana(mut self, v: Iana) -> Self {
        self.iana.push(v);
        self
    }

    /// Adds a calendar component; may be called more than once. At least
    /// one is REQUIRED by [`Self::build`].
    pub fn component(mut self, v: impl Into<Component>) -> Self {
        self.components.push(v.into());
        self
    }

    /// Validates the cross-field rules RFC 5545 §3.6 places on the whole
    /// `VCALENDAR` and assembles the finished [`Calendar`].
    pub fn build(self) -> Result<Calendar, ComponentError> {
        if self.components.is_empty() {
            return Err(ComponentError::RequiresAtLeastOne(
                "VCALENDAR",
                "calendar component",
            ));
        }
        // Each `Event` was built before this calendar's `METHOD` was known,
        // so its own `build(has_method)` can't have enforced this.
        if self.method.is_none()
            && self.components.iter().any(
                |c| matches!(c, Component::Event(e) if e.dtstart().is_none()),
            )
        {
            return Err(ComponentError::MissingField("DTSTART"));
        }
        validate_calendar(self.method.as_ref(), &self.components)?;

        Ok(Calendar {
            prodid: self.prodid,
            version: self.version,
            calscale: self.calscale,
            method: self.method,
            uid: self.uid,
            last_mod: self.last_mod,
            url: self.url,
            refresh_interval: self.refresh_interval,
            source: self.source,
            color: self.color,
            name: self.name,
            description: self.description,
            categories: self.categories,
            image: self.image,
            xprop: self.xprop,
            iana: self.iana,
            components: self.components,
        })
    }
}

impl TryFrom<CalendarBuilder> for Calendar {
    type Error = ComponentError;

    fn try_from(builder: CalendarBuilder) -> Result<Self, Self::Error> {
        builder.build()
    }
}

macro_rules! impl_from_component {
    ($($ty:ty => $variant:ident),* $(,)?) => {$(
        impl From<$ty> for Component {
            fn from(v: $ty) -> Self {
                Self::$variant(v)
            }
        }
    )*};
}

impl_from_component! {
    Event => Event,
    Todo => Todo,
    Journal => Journal,
    FreeBusy => FreeBusy,
    Timezone => Timezone,
    UnknownComponent => Unknown,
}

#[cfg(feature = "rfc-7953")]
impl_from_component!(Availability => Availability);

/// Error returned by [`Calendar::parse`] when an iCalendar byte stream fails
/// to parse, either because it can't be tokenized at all (`Lexer`) or
/// because the resulting tokens don't form a valid iCalendar object
/// (`Parse`).
#[derive(Debug, thiserror::Error)]
pub enum CalendarParseError {
    /// The raw input couldn't be tokenized.
    #[error(transparent)]
    Lexer(#[from] LexerError),
    /// The token stream didn't form a valid iCalendar object.
    #[error(transparent)]
    Parse(#[from] ParseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // rustfmt's `format_strings` wrapping of this literal corrupts its
    // runtime value (it splits inside a `\r\n` escape pair, not between
    // whole escapes) — skip it here rather than let a formatting pass
    // silently reintroduce that bug.
    #[rustfmt::skip]
    fn calendar_display_round_trips_a_minimal_calendar() {
        let src = b"BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:123@example.com\r\nDTSTAMP:19970901T130000Z\r\nDTSTART:19970903T163000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let calendar = Calendar::parse(src).unwrap();
        assert_eq!(
            calendar.to_string(),
            "BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nDTSTAMP:19970901T130000Z\r\nUID:123@example.com\r\nDTSTART:19970903T163000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
        );
    }

    #[test]
    fn calendar_display_output_of_a_todo_with_alarm_reparses_successfully() {
        // tests/fixtures/rfc5545/todo_with_alarm.ics — a VTODO with a
        // nested VALARM and a folded ATTACH line, to exercise nested
        // sub-component Display and nothing getting dropped along the way.
        let src = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/rfc5545/todo_with_alarm.ics"
        ));
        let original = Calendar::parse(src).unwrap();
        let rendered = original.to_string();

        let reparsed =
            Calendar::parse(rendered.as_bytes()).unwrap_or_else(|e| {
                panic!(
                    "rendered output failed to reparse: {e}\n---\n{rendered}"
                )
            });

        assert_eq!(reparsed.components().len(), 1);
        let Component::Todo(todo) = &reparsed.components()[0] else {
            panic!("expected a VTODO");
        };
        assert_eq!(todo.uid().as_str(), "uid4@example.com");
        assert_eq!(todo.alarms().len(), 1);
        // ACTION:AUDIO in the source fixture — confirms the nested VALARM's
        // own property survived Display + reparse, not just the VALARM
        // wrapper's presence.
        assert!(matches!(
            todo.alarms()[0].action().kind(),
            crate::properties::ActionEnum::Audio
        ));
    }

    #[test]
    fn calendar_display_output_of_a_timezone_meeting_reparses_successfully() {
        // tests/fixtures/rfc5545/group_meeting_with_timezone.ics — a
        // VTIMEZONE with both STANDARD and DAYLIGHT sub-components plus a
        // VEVENT whose DTSTART/DTEND carry a TZID param, to exercise the
        // STANDARD/DAYLIGHT naming split in `fmt_tz_observance` and the
        // TZID-resolved-to-UTC round-trip.
        let src = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/rfc5545/group_meeting_with_timezone.ics"
        ));
        let original = Calendar::parse(src).unwrap();
        let rendered = original.to_string();

        let reparsed =
            Calendar::parse(rendered.as_bytes()).unwrap_or_else(|e| {
                panic!(
                    "rendered output failed to reparse: {e}\n---\n{rendered}"
                )
            });

        assert_eq!(reparsed.components().len(), 2);
        let Component::Timezone(tz) = &reparsed.components()[0] else {
            panic!("expected a VTIMEZONE");
        };
        assert_eq!(tz.standardc().len(), 1);
        assert_eq!(tz.daylightc().len(), 1);

        let Component::Event(event) = &reparsed.components()[1] else {
            panic!("expected a VEVENT");
        };
        assert_eq!(event.uid().as_str(), "guid-1.example.com");
        assert_eq!(event.attendee().len(), 1);
    }
}
