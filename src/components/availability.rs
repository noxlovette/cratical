use crate::{
    components::{write_components, write_lines},
    properties::{
        BusyType, Categories, Classification, Comment, Contact,
        DateTimeCreated, DateTimeEnd, DateTimeStamp, DateTimeStart,
        Description, Duration, ExceptionDateTimes, Iana, LastModified,
        Location, Organizer, Priority, RRule, RecurrenceDateTimes,
        RecurrenceId, Sequence, Summary, Uid, UniformResourceLocator, Xprop,
    },
};

/// A "VAVAILABILITY" component indicates a period of time within which
/// availability information is provided.  A "VAVAILABILITY" component can
/// specify a start time and an end time or duration.  If "DTSTART" is not
/// present, then the start time is unbounded.  If "DTEND" or "DURATION"
/// are not present, then the end time is unbounded.  Within the specified
/// time period, availability defaults to a free-busy type of
/// "BUSY-UNAVAILABLE" (see [`BusyType`]), except for any time periods
/// corresponding to "AVAILABLE" subcomponents.
///
/// "AVAILABLE" subcomponents are used to indicate periods of free time
/// within the time range of the enclosing "VAVAILABILITY" component.
/// "AVAILABLE" subcomponents MAY include recurrence properties to specify
/// recurring periods of time, which can be overridden using normal
/// iCalendar recurrence behavior (i.e., use of the "RECURRENCE-ID"
/// property).
///
/// If specified, the "DTSTART" and "DTEND" properties in "VAVAILABILITY"
/// components and "AVAILABLE" subcomponents MUST be "DATE-TIME" values
/// specified as either the date with UTC time or the date with local time
/// and a time zone reference.
///
/// The iCalendar object containing the "VAVAILABILITY" component MUST
/// contain appropriate "VTIMEZONE" components corresponding to each unique
/// "TZID" parameter value used in any DATE-TIME properties in all
/// components, unless \[RFC7809\] is in effect.
///
/// When used to publish available time, the "ORGANIZER" property
/// specifies the calendar user associated with the published available
/// time.
///
/// If the "PRIORITY" property is specified in "VAVAILABILITY" components,
/// it is used to determine how that component is combined with other
/// "VAVAILABILITY" components.  See Section 4.
///
/// Other calendar properties MAY be specified in "VAVAILABILITY" or
/// "AVAILABLE" components and are considered attributes of the marked
/// block of time.  Their usage is application specific.
///
/// Example:  The following is an example of a "VAVAILABILITY" calendar
/// component used to represent the availability of a user, always
/// available Monday through Friday, 9:00 am to 5:00 pm in the
/// America/Montreal time zone:
///
/// > BEGIN:VAVAILABILITY
/// >
/// > ORGANIZER:mailto:bernard@example.com
/// >
/// > UID:0428C7D2-688E-4D2E-AC52-CD112E2469DF
/// >
/// > DTSTAMP:20111005T133225Z
/// >
/// > BEGIN:AVAILABLE
/// >
/// > UID:34EDA59B-6BB1-4E94-A66C-64999089C0AF
/// >
/// > SUMMARY:Monday to Friday from 9:00 to 17:00
/// >
/// > DTSTART;TZID=America/Montreal:20111002T090000
/// >
/// > DTEND;TZID=America/Montreal:20111002T170000
/// >
/// > RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR
/// >
/// > END:AVAILABLE
/// >
/// > END:VAVAILABILITY
/// >
///
/// [Section 3.1](https://datatracker.ietf.org/doc/html/rfc7953#section-3.1)
#[derive(Debug)]
pub struct Availability {
    pub(crate) dtstamp: DateTimeStamp,
    pub(crate) uid: Uid,
    pub(crate) busytype: Option<BusyType>,
    pub(crate) class: Option<Classification>,
    pub(crate) created: Option<DateTimeCreated>,
    pub(crate) description: Option<Description>,
    pub(crate) dtstart: Option<DateTimeStart>,
    pub(crate) last_mod: Option<LastModified>,
    pub(crate) location: Option<Location>,
    pub(crate) organizer: Option<Organizer>,
    pub(crate) priority: Option<Priority>,
    pub(crate) seq: Option<Sequence>,
    pub(crate) summary: Option<Summary>,
    pub(crate) url: Option<UniformResourceLocator>,
    pub(crate) dtend: Option<DateTimeEnd>,
    pub(crate) duration: Option<Duration>,
    pub(crate) categories: Vec<Categories>,
    pub(crate) comment: Vec<Comment>,
    pub(crate) contact: Vec<Contact>,
    pub(crate) xprop: Vec<Xprop>,
    pub(crate) iana: Vec<Iana>,
    pub(crate) available: Vec<Available>,
}

impl Availability {
    /// The `DTSTAMP` property.
    pub fn dtstamp(&self) -> &DateTimeStamp {
        &self.dtstamp
    }

    /// The `UID` property.
    pub fn uid(&self) -> &Uid {
        &self.uid
    }

    /// The `BUSYTYPE` property, if present.
    pub fn busytype(&self) -> Option<&BusyType> {
        self.busytype.as_ref()
    }

    /// The `CLASS` property, if present.
    pub fn class(&self) -> Option<&Classification> {
        self.class.as_ref()
    }

    /// The `CREATED` property, if present.
    pub fn created(&self) -> Option<&DateTimeCreated> {
        self.created.as_ref()
    }

    /// The `DESCRIPTION` property, if present.
    pub fn description(&self) -> Option<&Description> {
        self.description.as_ref()
    }

    /// The `DTSTART` property, if present.
    pub fn dtstart(&self) -> Option<&DateTimeStart> {
        self.dtstart.as_ref()
    }

    /// The `LAST-MODIFIED` property, if present.
    pub fn last_mod(&self) -> Option<&LastModified> {
        self.last_mod.as_ref()
    }

    /// The `LOCATION` property, if present.
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }

    /// The `ORGANIZER` property, if present.
    pub fn organizer(&self) -> Option<&Organizer> {
        self.organizer.as_ref()
    }

    /// The `PRIORITY` property, if present.
    pub fn priority(&self) -> Option<&Priority> {
        self.priority.as_ref()
    }

    /// The `SEQUENCE` property, if present.
    pub fn seq(&self) -> Option<&Sequence> {
        self.seq.as_ref()
    }

    /// The `SUMMARY` property, if present.
    pub fn summary(&self) -> Option<&Summary> {
        self.summary.as_ref()
    }

    /// The `URL` property, if present.
    pub fn url(&self) -> Option<&UniformResourceLocator> {
        self.url.as_ref()
    }

    /// The `DTEND` property, if present.
    pub fn dtend(&self) -> Option<&DateTimeEnd> {
        self.dtend.as_ref()
    }

    /// The `DURATION` property, if present.
    pub fn duration(&self) -> Option<&Duration> {
        self.duration.as_ref()
    }

    /// The `CATEGORIES` properties.
    pub fn categories(&self) -> &[Categories] {
        &self.categories
    }

    /// The `COMMENT` properties.
    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    /// The `CONTACT` properties.
    pub fn contact(&self) -> &[Contact] {
        &self.contact
    }

    /// The non-standard (`X-`) properties.
    pub fn xprop(&self) -> &[Xprop] {
        &self.xprop
    }

    /// The IANA-registered properties this crate doesn't otherwise model.
    pub fn iana(&self) -> &[Iana] {
        &self.iana
    }

    /// The nested `AVAILABLE` subcomponents.
    pub fn available(&self) -> &[Available] {
        &self.available
    }
}

impl std::fmt::Display for Availability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:VAVAILABILITY\r\n")?;
        write!(f, "{}\r\n", self.dtstamp)?;
        write!(f, "{}\r\n", self.uid)?;
        if let Some(v) = &self.busytype {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.class {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.created {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.description {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.dtstart {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.last_mod {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.location {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.organizer {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.priority {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.seq {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.summary {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.url {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.dtend {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.duration {
            write!(f, "{v}\r\n")?;
        }
        write_lines(f, &self.categories)?;
        write_lines(f, &self.comment)?;
        write_lines(f, &self.contact)?;
        write_lines(f, &self.xprop)?;
        write_lines(f, &self.iana)?;
        write_components(f, &self.available)?;
        write!(f, "END:VAVAILABILITY\r\n")
    }
}

/// An "AVAILABLE" subcomponent, nested inside a [`Availability`]
/// (`VAVAILABILITY`) component.  See [`Availability`]'s own docs for the
/// full description of how "AVAILABLE" subcomponents relate to their
/// enclosing "VAVAILABILITY" component.
///
/// Example:
///
/// > BEGIN:AVAILABLE
/// >
/// > UID:34EDA59B-6BB1-4E94-A66C-64999089C0AF
/// >
/// > SUMMARY:Monday to Friday from 9:00 to 17:00
/// >
/// > DTSTART;TZID=America/Montreal:20111002T090000
/// >
/// > DTEND;TZID=America/Montreal:20111002T170000
/// >
/// > RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR
/// >
/// > END:AVAILABLE
/// >
///
/// **Deviation from the formal grammar:** RFC 7953's `availableprop` ABNF
/// marks `DTSTAMP` REQUIRED on `AVAILABLE`, but every worked example in the
/// RFC itself — §3.1's own three examples, reproduced verbatim above and in
/// this crate's vendored `tests/fixtures/collective-icalendar/availabilities/`
/// fixtures — omits it. No IETF errata exists for this inconsistency. This
/// crate follows the RFC's own real-world usage over its formal grammar
/// here: `DTSTAMP` is modeled as optional.
///
/// [Section 3.1](https://datatracker.ietf.org/doc/html/rfc7953#section-3.1)
#[derive(Debug)]
pub struct Available {
    pub(crate) dtstamp: Option<DateTimeStamp>,
    pub(crate) dtstart: DateTimeStart,
    pub(crate) uid: Uid,
    pub(crate) created: Option<DateTimeCreated>,
    pub(crate) description: Option<Description>,
    pub(crate) last_mod: Option<LastModified>,
    pub(crate) location: Option<Location>,
    pub(crate) recurid: Option<RecurrenceId>,
    pub(crate) rrule: Option<RRule>,
    pub(crate) summary: Option<Summary>,
    pub(crate) dtend: Option<DateTimeEnd>,
    pub(crate) duration: Option<Duration>,
    pub(crate) categories: Vec<Categories>,
    pub(crate) comment: Vec<Comment>,
    pub(crate) contact: Vec<Contact>,
    pub(crate) exdate: Vec<ExceptionDateTimes>,
    pub(crate) rdate: Vec<RecurrenceDateTimes>,
    pub(crate) xprop: Vec<Xprop>,
    pub(crate) iana: Vec<Iana>,
}

impl Available {
    /// The `DTSTAMP` property, if present. Modeled as optional despite the
    /// formal grammar marking it REQUIRED — see this type's own docs.
    pub fn dtstamp(&self) -> Option<&DateTimeStamp> {
        self.dtstamp.as_ref()
    }

    /// The `DTSTART` property.
    pub fn dtstart(&self) -> &DateTimeStart {
        &self.dtstart
    }

    /// The `UID` property.
    pub fn uid(&self) -> &Uid {
        &self.uid
    }

    /// The `CREATED` property, if present.
    pub fn created(&self) -> Option<&DateTimeCreated> {
        self.created.as_ref()
    }

    /// The `DESCRIPTION` property, if present.
    pub fn description(&self) -> Option<&Description> {
        self.description.as_ref()
    }

    /// The `LAST-MODIFIED` property, if present.
    pub fn last_mod(&self) -> Option<&LastModified> {
        self.last_mod.as_ref()
    }

    /// The `LOCATION` property, if present.
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }

    /// The `RECURRENCE-ID` property, if present.
    pub fn recurid(&self) -> Option<&RecurrenceId> {
        self.recurid.as_ref()
    }

    /// The `RRULE` property, if present.
    pub fn rrule(&self) -> Option<&RRule> {
        self.rrule.as_ref()
    }

    /// The `SUMMARY` property, if present.
    pub fn summary(&self) -> Option<&Summary> {
        self.summary.as_ref()
    }

    /// The `DTEND` property, if present.
    pub fn dtend(&self) -> Option<&DateTimeEnd> {
        self.dtend.as_ref()
    }

    /// The `DURATION` property, if present.
    pub fn duration(&self) -> Option<&Duration> {
        self.duration.as_ref()
    }

    /// The `CATEGORIES` properties.
    pub fn categories(&self) -> &[Categories] {
        &self.categories
    }

    /// The `COMMENT` properties.
    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    /// The `CONTACT` properties.
    pub fn contact(&self) -> &[Contact] {
        &self.contact
    }

    /// The `EXDATE` properties.
    pub fn exdate(&self) -> &[ExceptionDateTimes] {
        &self.exdate
    }

    /// The `RDATE` properties.
    pub fn rdate(&self) -> &[RecurrenceDateTimes] {
        &self.rdate
    }

    /// The non-standard (`X-`) properties.
    pub fn xprop(&self) -> &[Xprop] {
        &self.xprop
    }

    /// The IANA-registered properties this crate doesn't otherwise model.
    pub fn iana(&self) -> &[Iana] {
        &self.iana
    }
}

impl std::fmt::Display for Available {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:AVAILABLE\r\n")?;
        if let Some(v) = &self.dtstamp {
            write!(f, "{v}\r\n")?;
        }
        write!(f, "{}\r\n", self.dtstart)?;
        write!(f, "{}\r\n", self.uid)?;
        if let Some(v) = &self.created {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.description {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.last_mod {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.location {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.recurid {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.rrule {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.summary {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.dtend {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.duration {
            write!(f, "{v}\r\n")?;
        }
        write_lines(f, &self.categories)?;
        write_lines(f, &self.comment)?;
        write_lines(f, &self.contact)?;
        write_lines(f, &self.exdate)?;
        write_lines(f, &self.rdate)?;
        write_lines(f, &self.xprop)?;
        write_lines(f, &self.iana)?;
        write!(f, "END:AVAILABLE\r\n")
    }
}
