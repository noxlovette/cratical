use std::marker::PhantomData;

use crate::{
    ast::ComponentError,
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

/// [`AvailabilityBuilder`]'s state before either `DTEND` or `DURATION` has
/// been set.
#[derive(Debug)]
pub struct Unset;

/// [`AvailabilityBuilder`]'s state once `DTEND` has been set —
/// `.duration()` no longer exists on the builder in this state.
#[derive(Debug)]
pub struct HasDtend;

/// [`AvailabilityBuilder`]'s state once `DURATION` has been set —
/// `.dtend()` no longer exists on the builder in this state.
#[derive(Debug)]
pub struct HasDuration;

/// Builder for [`Availability`]. Enforces RFC 7953 §3.1's `DTEND`/
/// `DURATION` mutual exclusion at compile time via the type-state pattern
/// — see [`crate::components::event::EventBuilder`]'s doc comment for how
/// the state machine works; it's the same shape here. `DURATION`
/// additionally requires `DTSTART` to also be present, which — like the
/// value-type matching between `DTSTART` and `DTEND` — can't be resolved
/// until the actual property values are known, so both stay runtime
/// checks in [`Self::build`].
#[derive(Debug)]
pub struct AvailabilityBuilder<S = Unset> {
    dtstamp: DateTimeStamp,
    uid: Uid,
    busytype: Option<BusyType>,
    class: Option<Classification>,
    created: Option<DateTimeCreated>,
    description: Option<Description>,
    dtstart: Option<DateTimeStart>,
    last_mod: Option<LastModified>,
    location: Option<Location>,
    organizer: Option<Organizer>,
    priority: Option<Priority>,
    seq: Option<Sequence>,
    summary: Option<Summary>,
    url: Option<UniformResourceLocator>,
    dtend: Option<DateTimeEnd>,
    duration: Option<Duration>,
    categories: Vec<Categories>,
    comment: Vec<Comment>,
    contact: Vec<Contact>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
    available: Vec<Available>,
    _state: PhantomData<S>,
}

impl<S> AvailabilityBuilder<S> {
    fn retag<S2>(self) -> AvailabilityBuilder<S2> {
        AvailabilityBuilder {
            dtstamp: self.dtstamp,
            uid: self.uid,
            busytype: self.busytype,
            class: self.class,
            created: self.created,
            description: self.description,
            dtstart: self.dtstart,
            last_mod: self.last_mod,
            location: self.location,
            organizer: self.organizer,
            priority: self.priority,
            seq: self.seq,
            summary: self.summary,
            url: self.url,
            dtend: self.dtend,
            duration: self.duration,
            categories: self.categories,
            comment: self.comment,
            contact: self.contact,
            xprop: self.xprop,
            iana: self.iana,
            available: self.available,
            _state: PhantomData,
        }
    }
}

impl AvailabilityBuilder<Unset> {
    /// Starts building a `VAVAILABILITY` from its two properties RFC 7953
    /// §3.1 requires unconditionally: `DTSTAMP` and `UID`.
    pub fn new(dtstamp: DateTimeStamp, uid: Uid) -> Self {
        Self {
            dtstamp,
            uid,
            busytype: None,
            class: None,
            created: None,
            description: None,
            dtstart: None,
            last_mod: None,
            location: None,
            organizer: None,
            priority: None,
            seq: None,
            summary: None,
            url: None,
            dtend: None,
            duration: None,
            categories: Vec::new(),
            comment: Vec::new(),
            contact: Vec::new(),
            xprop: Vec::new(),
            iana: Vec::new(),
            available: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Sets `DTEND` (RFC 7953 §3.1). Mutually exclusive with
    /// [`Self::duration`] — moves the builder into [`HasDtend`], on which
    /// `.duration()` doesn't exist.
    pub fn dtend(self, dtend: DateTimeEnd) -> AvailabilityBuilder<HasDtend> {
        AvailabilityBuilder {
            dtend: Some(dtend),
            ..self.retag()
        }
    }

    /// Sets `DURATION` (RFC 7953 §3.1). Mutually exclusive with
    /// [`Self::dtend`] — moves the builder into [`HasDuration`], on which
    /// `.dtend()` doesn't exist. RFC 7953 also requires `DTSTART` to be set
    /// whenever `DURATION` is — checked in [`Self::build`].
    pub fn duration(
        self,
        duration: Duration,
    ) -> AvailabilityBuilder<HasDuration> {
        AvailabilityBuilder {
            duration: Some(duration),
            ..self.retag()
        }
    }
}

impl<S> AvailabilityBuilder<S> {
    /// Sets `BUSYTYPE`.
    pub fn busytype(mut self, v: BusyType) -> Self {
        self.busytype = Some(v);
        self
    }

    /// Sets `CLASS`.
    pub fn class(mut self, v: Classification) -> Self {
        self.class = Some(v);
        self
    }

    /// Sets `CREATED`.
    pub fn created(mut self, v: DateTimeCreated) -> Self {
        self.created = Some(v);
        self
    }

    /// Sets `DESCRIPTION`.
    pub fn description(mut self, v: Description) -> Self {
        self.description = Some(v);
        self
    }

    /// Sets `DTSTART`.
    pub fn dtstart(mut self, v: DateTimeStart) -> Self {
        self.dtstart = Some(v);
        self
    }

    /// Sets `LAST-MODIFIED`.
    pub fn last_mod(mut self, v: LastModified) -> Self {
        self.last_mod = Some(v);
        self
    }

    /// Sets `LOCATION`.
    pub fn location(mut self, v: Location) -> Self {
        self.location = Some(v);
        self
    }

    /// Sets `ORGANIZER`.
    pub fn organizer(mut self, v: Organizer) -> Self {
        self.organizer = Some(v);
        self
    }

    /// Sets `PRIORITY`.
    pub fn priority(mut self, v: Priority) -> Self {
        self.priority = Some(v);
        self
    }

    /// Sets `SEQUENCE`.
    pub fn seq(mut self, v: Sequence) -> Self {
        self.seq = Some(v);
        self
    }

    /// Sets `SUMMARY`.
    pub fn summary(mut self, v: Summary) -> Self {
        self.summary = Some(v);
        self
    }

    /// Sets `URL`.
    pub fn url(mut self, v: UniformResourceLocator) -> Self {
        self.url = Some(v);
        self
    }

    /// Adds a `CATEGORIES` property.
    pub fn categories(mut self, v: Categories) -> Self {
        self.categories.push(v);
        self
    }

    /// Adds a `COMMENT` property.
    pub fn comment(mut self, v: Comment) -> Self {
        self.comment.push(v);
        self
    }

    /// Adds a `CONTACT` property.
    pub fn contact(mut self, v: Contact) -> Self {
        self.contact.push(v);
        self
    }

    /// Adds a non-standard (`X-`) property.
    pub fn xprop(mut self, v: Xprop) -> Self {
        self.xprop.push(v);
        self
    }

    /// Adds an IANA-registered property this crate doesn't otherwise model.
    pub fn iana(mut self, v: Iana) -> Self {
        self.iana.push(v);
        self
    }

    /// Adds an `AVAILABLE` sub-component, already built via
    /// [`AvailableBuilder`].
    pub fn available(mut self, v: Available) -> Self {
        self.available.push(v);
        self
    }

    /// Validates the cross-field rules RFC 7953 §3.1 places on
    /// `VAVAILABILITY` and assembles the finished [`Availability`].
    pub fn build(self) -> Result<Availability, ComponentError> {
        if self.duration.is_some() && self.dtstart.is_none() {
            return Err(ComponentError::Requires("DURATION", "DTSTART"));
        }
        if let Some(dtstart) = self.dtstart.as_ref() {
            dtstart.cmp_value_type(
                self.dtend.as_ref().map(DateTimeEnd::value),
                "DTEND",
            )?;
        }

        Ok(Availability {
            dtstamp: self.dtstamp,
            uid: self.uid,
            busytype: self.busytype,
            class: self.class,
            created: self.created,
            description: self.description,
            dtstart: self.dtstart,
            last_mod: self.last_mod,
            location: self.location,
            organizer: self.organizer,
            priority: self.priority,
            seq: self.seq,
            summary: self.summary,
            url: self.url,
            dtend: self.dtend,
            duration: self.duration,
            categories: self.categories,
            comment: self.comment,
            contact: self.contact,
            xprop: self.xprop,
            iana: self.iana,
            available: self.available,
        })
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

/// [`AvailableBuilder`]'s state before either `DTEND` or `DURATION` has
/// been set.
#[derive(Debug)]
pub struct AvailableUnset;

/// [`AvailableBuilder`]'s state once `DTEND` has been set — `.duration()`
/// no longer exists on the builder in this state.
#[derive(Debug)]
pub struct AvailableHasDtend;

/// [`AvailableBuilder`]'s state once `DURATION` has been set — `.dtend()`
/// no longer exists on the builder in this state.
#[derive(Debug)]
pub struct AvailableHasDuration;

/// Builder for [`Available`]. Enforces RFC 7953 §3.1's `DTEND`/`DURATION`
/// mutual exclusion at compile time via the type-state pattern — see
/// [`crate::components::event::EventBuilder`]'s doc comment for how the
/// state machine works; it's the same shape here. The value-type/`TZID`
/// matching between `DTSTART` and its siblings can't be resolved until the
/// actual property values are known, so it stays a runtime check in
/// [`Self::build`].
#[derive(Debug)]
pub struct AvailableBuilder<S = AvailableUnset> {
    dtstamp: Option<DateTimeStamp>,
    dtstart: DateTimeStart,
    uid: Uid,
    created: Option<DateTimeCreated>,
    description: Option<Description>,
    last_mod: Option<LastModified>,
    location: Option<Location>,
    recurid: Option<RecurrenceId>,
    rrule: Option<RRule>,
    summary: Option<Summary>,
    dtend: Option<DateTimeEnd>,
    duration: Option<Duration>,
    categories: Vec<Categories>,
    comment: Vec<Comment>,
    contact: Vec<Contact>,
    exdate: Vec<ExceptionDateTimes>,
    rdate: Vec<RecurrenceDateTimes>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
    _state: PhantomData<S>,
}

impl<S> AvailableBuilder<S> {
    fn retag<S2>(self) -> AvailableBuilder<S2> {
        AvailableBuilder {
            dtstamp: self.dtstamp,
            dtstart: self.dtstart,
            uid: self.uid,
            created: self.created,
            description: self.description,
            last_mod: self.last_mod,
            location: self.location,
            recurid: self.recurid,
            rrule: self.rrule,
            summary: self.summary,
            dtend: self.dtend,
            duration: self.duration,
            categories: self.categories,
            comment: self.comment,
            contact: self.contact,
            exdate: self.exdate,
            rdate: self.rdate,
            xprop: self.xprop,
            iana: self.iana,
            _state: PhantomData,
        }
    }
}

impl AvailableBuilder<AvailableUnset> {
    /// Starts building an `AVAILABLE` from its two required properties,
    /// `DTSTART` and `UID` (RFC 7953 §3.1; `DTSTAMP` is REQUIRED by the
    /// formal grammar but modeled as optional here — see [`Available`]'s
    /// own docs).
    pub fn new(dtstart: DateTimeStart, uid: Uid) -> Self {
        Self {
            dtstamp: None,
            dtstart,
            uid,
            created: None,
            description: None,
            last_mod: None,
            location: None,
            recurid: None,
            rrule: None,
            summary: None,
            dtend: None,
            duration: None,
            categories: Vec::new(),
            comment: Vec::new(),
            contact: Vec::new(),
            exdate: Vec::new(),
            rdate: Vec::new(),
            xprop: Vec::new(),
            iana: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Sets `DTEND` (RFC 7953 §3.1). Mutually exclusive with
    /// [`Self::duration`] — moves the builder into [`AvailableHasDtend`],
    /// on which `.duration()` doesn't exist.
    pub fn dtend(
        self,
        dtend: DateTimeEnd,
    ) -> AvailableBuilder<AvailableHasDtend> {
        AvailableBuilder {
            dtend: Some(dtend),
            ..self.retag()
        }
    }

    /// Sets `DURATION` (RFC 7953 §3.1). Mutually exclusive with
    /// [`Self::dtend`] — moves the builder into [`AvailableHasDuration`],
    /// on which `.dtend()` doesn't exist.
    pub fn duration(
        self,
        duration: Duration,
    ) -> AvailableBuilder<AvailableHasDuration> {
        AvailableBuilder {
            duration: Some(duration),
            ..self.retag()
        }
    }
}

impl<S> AvailableBuilder<S> {
    /// Sets `DTSTAMP`.
    pub fn dtstamp(mut self, v: DateTimeStamp) -> Self {
        self.dtstamp = Some(v);
        self
    }

    /// Sets `CREATED`.
    pub fn created(mut self, v: DateTimeCreated) -> Self {
        self.created = Some(v);
        self
    }

    /// Sets `DESCRIPTION`.
    pub fn description(mut self, v: Description) -> Self {
        self.description = Some(v);
        self
    }

    /// Sets `LAST-MODIFIED`.
    pub fn last_mod(mut self, v: LastModified) -> Self {
        self.last_mod = Some(v);
        self
    }

    /// Sets `LOCATION`.
    pub fn location(mut self, v: Location) -> Self {
        self.location = Some(v);
        self
    }

    /// Sets `RECURRENCE-ID`.
    pub fn recurid(mut self, v: RecurrenceId) -> Self {
        self.recurid = Some(v);
        self
    }

    /// Sets `RRULE`.
    pub fn rrule(mut self, v: RRule) -> Self {
        self.rrule = Some(v);
        self
    }

    /// Sets `SUMMARY`.
    pub fn summary(mut self, v: Summary) -> Self {
        self.summary = Some(v);
        self
    }

    /// Adds a `CATEGORIES` property.
    pub fn categories(mut self, v: Categories) -> Self {
        self.categories.push(v);
        self
    }

    /// Adds a `COMMENT` property.
    pub fn comment(mut self, v: Comment) -> Self {
        self.comment.push(v);
        self
    }

    /// Adds a `CONTACT` property.
    pub fn contact(mut self, v: Contact) -> Self {
        self.contact.push(v);
        self
    }

    /// Adds an `EXDATE` property.
    pub fn exdate(mut self, v: ExceptionDateTimes) -> Self {
        self.exdate.push(v);
        self
    }

    /// Adds an `RDATE` property.
    pub fn rdate(mut self, v: RecurrenceDateTimes) -> Self {
        self.rdate.push(v);
        self
    }

    /// Adds a non-standard (`X-`) property.
    pub fn xprop(mut self, v: Xprop) -> Self {
        self.xprop.push(v);
        self
    }

    /// Adds an IANA-registered property this crate doesn't otherwise model.
    pub fn iana(mut self, v: Iana) -> Self {
        self.iana.push(v);
        self
    }

    /// Validates the cross-field rules RFC 7953 §3.1 places on `AVAILABLE`
    /// and assembles the finished [`Available`].
    pub fn build(self) -> Result<Available, ComponentError> {
        self.dtstart.cmp_until(self.rrule.as_ref())?;
        self.dtstart.cmp_value_type(
            self.dtend.as_ref().map(DateTimeEnd::value),
            "DTEND",
        )?;
        self.dtstart.cmp_exdate(&self.exdate)?;
        self.dtstart.cmp_rdate(&self.rdate)?;
        self.dtstart.cmp_exdate_tzid(&self.exdate)?;
        self.dtstart.cmp_rdate_tzid(&self.rdate)?;

        Ok(Available {
            dtstamp: self.dtstamp,
            dtstart: self.dtstart,
            uid: self.uid,
            created: self.created,
            description: self.description,
            last_mod: self.last_mod,
            location: self.location,
            recurid: self.recurid,
            rrule: self.rrule,
            summary: self.summary,
            dtend: self.dtend,
            duration: self.duration,
            categories: self.categories,
            comment: self.comment,
            contact: self.contact,
            exdate: self.exdate,
            rdate: self.rdate,
            xprop: self.xprop,
            iana: self.iana,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::{DateOrDatetime, DateTime};
    use chrono::{TimeZone, Utc};

    fn dtstamp() -> DateTimeStamp {
        DateTimeStamp::new(DateTime::Utc(
            Utc.with_ymd_and_hms(2011, 10, 5, 13, 32, 25).unwrap(),
        ))
    }

    #[test]
    fn availability_builder_round_trips_a_minimal_availability() {
        let availability = AvailabilityBuilder::new(
            dtstamp(),
            Uid::new("0428C7D2-688E-4D2E-AC52-CD112E2469DF".into()),
        )
        .build()
        .unwrap();
        assert_eq!(
            availability.to_string(),
            "BEGIN:VAVAILABILITY\r\nDTSTAMP:20111005T133225Z\r\nUID:\
             0428C7D2-688E-4D2E-AC52-CD112E2469DF\r\nEND:VAVAILABILITY\r\n"
        );
    }

    #[test]
    fn availability_builder_rejects_duration_without_dtstart() {
        let result = AvailabilityBuilder::new(
            dtstamp(),
            Uid::new("uid@example.com".into()),
        )
        .duration(Duration::new(crate::values::Duration::new(
            chrono::Duration::hours(1),
        )))
        .build();
        assert!(matches!(
            result,
            Err(ComponentError::Requires("DURATION", "DTSTART"))
        ));
    }

    #[test]
    fn available_builder_round_trips_a_minimal_available() {
        let dtstart = crate::properties::DateTimeStartBuilder::new(
            DateOrDatetime::DateTime(DateTime::Utc(
                Utc.with_ymd_and_hms(2011, 10, 2, 9, 0, 0).unwrap(),
            )),
        )
        .build();
        let available = AvailableBuilder::new(
            dtstart,
            Uid::new("34EDA59B-6BB1-4E94-A66C-64999089C0AF".into()),
        )
        .build()
        .unwrap();
        assert_eq!(
            available.to_string(),
            "BEGIN:AVAILABLE\r\nDTSTART:20111002T090000Z\r\nUID:\
             34EDA59B-6BB1-4E94-A66C-64999089C0AF\r\nEND:AVAILABLE\r\n"
        );
    }
}
