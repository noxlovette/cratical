use crate::{
    ast::{
        Lexer, LexerError,
        parser::{ParseError, Parser},
    },
    components::{
        event::Event, free_busy::FreeBusy, journal::Journal,
        timezone::Timezone, todo::Todo, unknown::UnknownComponent, write_lines,
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

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Event(c) => write!(f, "{c}"),
            Self::Todo(c) => write!(f, "{c}"),
            Self::Journal(c) => write!(f, "{c}"),
            Self::FreeBusy(c) => write!(f, "{c}"),
            Self::Timezone(c) => write!(f, "{c}"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_display_round_trips_a_minimal_calendar() {
        let src = b"BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:123@example.com\r\nDTSTAMP:19970901T130000Z\r\nDTSTART:19970903T163000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let calendar = Calendar::parse(src).unwrap();
        assert_eq!(
            calendar.to_string(),
            "BEGIN:VCALENDAR\r\n\
             PRODID:-//example//EN\r\n\
             VERSION:2.0\r\n\
             BEGIN:VEVENT\r\n\
             DTSTAMP:19970901T130000Z\r\n\
             UID:123@example.com\r\n\
             DTSTART:19970903T163000Z\r\n\
             END:VEVENT\r\n\
             END:VCALENDAR\r\n"
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

        let reparsed = Calendar::parse(rendered.as_bytes())
            .unwrap_or_else(|e| panic!("rendered output failed to reparse: {e}\n---\n{rendered}"));

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

        let reparsed = Calendar::parse(rendered.as_bytes())
            .unwrap_or_else(|e| panic!("rendered output failed to reparse: {e}\n---\n{rendered}"));

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
