use crate::{
    ast::{
        Lexer, LexerError,
        parser::{ParseError, Parser},
    },
    components::{
        event::Event, free_busy::FreeBusy, journal::Journal,
        timezone::Timezone, todo::Todo, unknown::UnknownComponent,
    },
    properties::{
        CalendarScale, Iana, Method, ProductIdentifier, Version, Xprop,
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
    /// An unrecognized `iana-comp`/`x-comp` (RFC 5545 §3.6), preserved
    /// verbatim rather than dropped. See [`UnknownComponent`].
    Unknown(UnknownComponent),
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
    /// errors if anything follows.
    pub fn parse(src: &[u8]) -> Result<Self, CalendarParseError> {
        let tokens = Lexer::new(src).scan()?;
        Parser::new(tokens).calendar().map_err(Into::into)
    }
}

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
