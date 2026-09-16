use std::marker::PhantomData;

use crate::{
    ast::ComponentError,
    components::{alarm::Alarm, write_components, write_lines},
    properties::{
        Attachment, Attendee, Categories, Classification, Color, Comment,
        Conference, Contact, DateTimeCreated, DateTimeEnd, DateTimeStamp,
        DateTimeStart, Description, Duration, ExceptionDateTimes, Geo, Iana,
        Image, LastModified, Location, Organizer, Priority, RRule,
        RecurrenceDateTimes, RecurrenceId, RelatedTo, RequestStatus, Resources,
        Sequence, Status, Summary, TimeTransparency, Uid,
        UniformResourceLocator, Xprop,
    },
};

/// A "VEVENT" calendar component is a grouping of
/// component properties, possibly including "VALARM" calendar
/// components, that represents a scheduled amount of time on a
/// calendar.  For example, it can be an activity; such as a one-hour
/// long, department meeting from 8:00 AM to 9:00 AM, tomorrow.
/// Generally, an event will take up time on an individual calendar.
/// Hence, the event will appear as an opaque interval in a search for
/// busy time.  Alternately, the event can have its Time Transparency
/// set to "TRANSPARENT" in order to prevent blocking of the event in
/// searches for busy time.
///
/// The "VEVENT" is also the calendar component used to specify an
/// anniversary or daily reminder within a calendar.  These events
/// have a DATE value type for the "DTSTART" property instead of the
/// default value type of DATE-TIME.  If such a "VEVENT" has a "DTEND"
/// property, it MUST be specified as a DATE value also.  The
/// anniversary type of "VEVENT" can span more than one date (i.e.,
/// "DTEND" property value is set to a calendar date after the
/// "DTSTART" property value).  If such a "VEVENT" has a "DURATION"
/// property, it MUST be specified as a "dur-day" or "dur-week" value.
///
/// The "DTSTART" property for a "VEVENT" specifies the inclusive
/// start of the event.  For recurring events, it also specifies the
/// very first instance in the recurrence set.  The "DTEND" property
/// for a "VEVENT" calendar component specifies the non-inclusive end
/// of the event.  For cases where a "VEVENT" calendar component
/// specifies a "DTSTART" property with a DATE value type but no
/// "DTEND" nor "DURATION" property, the event's duration is taken to
/// be one day.  For cases where a "VEVENT" calendar component
/// specifies a "DTSTART" property with a DATE-TIME value type but no
/// "DTEND" property, the event ends on the same calendar date and
/// time of day specified by the "DTSTART" property.
///
/// The "VEVENT" calendar component cannot be nested within another
/// calendar component.  However, "VEVENT" calendar components can be
/// related to each other or to a "VTODO" or to a "VJOURNAL" calendar
/// component with the "RELATED-TO" property.
///
///
/// Example:  The following is an example of the "VEVENT" calendar
/// component used to represent a meeting that will also be opaque to
/// searches for busy time:
///
/// > BEGIN:VEVENT
/// >
/// > UID:19970901T130000Z-123401@example.com
/// >
/// > DTSTAMP:19970901T130000Z
/// >
/// > DTSTART:19970903T163000Z
/// >
/// > DTEND:19970903T190000Z
/// >
/// > SUMMARY:Annual Employee Review
/// >
/// > CLASS:PRIVATE
/// >
/// > CATEGORIES:BUSINESS,HUMAN RESOURCES
/// >
/// > END:VEVENT
/// >
///
/// The following is an example of the "VEVENT" calendar component
/// used to represent a reminder that will not be opaque, but rather
/// transparent, to searches for busy time:
///
/// > BEGIN:VEVENT
/// >
/// > UID:19970901T130000Z-123402@example.com
/// >
/// > DTSTAMP:19970901T130000Z
/// >
/// > DTSTART:19970401T163000Z
/// >
/// > DTEND:19970402T010000Z
/// >
/// > SUMMARY:Laurel is in sensitivity awareness class.
/// >
/// > CLASS:PUBLIC
/// >
/// > CATEGORIES:BUSINESS,HUMAN RESOURCES
/// >
/// > TRANSP:TRANSPARENT
/// >
/// > END:VEVENT
/// >
///
/// The following is an example of the "VEVENT" calendar component
/// used to represent an anniversary that will occur annually:
///
/// > BEGIN:VEVENT
/// >
/// > UID:19970901T130000Z-123403@example.com
/// >
/// > DTSTAMP:19970901T130000Z
/// >
/// > DTSTART;VALUE=DATE:19971102
/// >
/// > SUMMARY:Our Blissful Anniversary
/// >
/// > TRANSP:TRANSPARENT
/// >
/// > CLASS:CONFIDENTIAL
/// >
/// > CATEGORIES:ANNIVERSARY,PERSONAL,SPECIAL OCCASION
/// >
/// > RRULE:FREQ=YEARLY
/// >
/// > END:VEVENT
/// >
/// The following is an example of the "VEVENT" calendar component
/// used to represent a multi-day event scheduled from June 28th, 2007
/// to July 8th, 2007 inclusively.  Note that the "DTEND" property is
/// set to July 9th, 2007, since the "DTEND" property specifies the
/// non-inclusive end of the event.
///
/// > BEGIN:VEVENT
/// >
/// > UID:20070423T123432Z-541111@example.com
/// >
/// > DTSTAMP:20070423T123432Z
/// >
/// > DTSTART;VALUE=DATE:20070628
/// >
/// > DTEND;VALUE=DATE:20070709
/// >
/// > SUMMARY:Festival International de Jazz de Montreal
/// >
/// > TRANSP:TRANSPARENT
/// >
/// > END:VEVENT
/// >
///
/// [Section 3.6.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6.1)
#[derive(Debug)]
pub struct Event {
    pub(crate) dtstamp: DateTimeStamp,
    pub(crate) uid: Uid,
    /// The following is REQUIRED if the component
    /// appears in an iCalendar object that doesn't
    /// specify the "METHOD" property; otherwise, it
    /// is OPTIONAL; in any case, it MUST NOT occur
    /// more than once.
    pub(crate) dtstart: Option<DateTimeStart>,
    pub(crate) class: Option<Classification>,
    pub(crate) created: Option<DateTimeCreated>,
    pub(crate) description: Option<Description>,
    pub(crate) geo: Option<Geo>,
    pub(crate) last_mod: Option<LastModified>,
    pub(crate) location: Option<Location>,
    pub(crate) organizer: Option<Organizer>,
    pub(crate) priority: Option<Priority>,
    pub(crate) seq: Option<Sequence>,
    pub(crate) status: Option<Status>,
    pub(crate) summary: Option<Summary>,
    pub(crate) transp: Option<TimeTransparency>,
    pub(crate) url: Option<UniformResourceLocator>,
    pub(crate) recurid: Option<RecurrenceId>,
    pub(crate) rrule: Option<RRule>,
    pub(crate) dtend: Option<DateTimeEnd>,
    pub(crate) duration: Option<Duration>,
    pub(crate) attach: Vec<Attachment>,
    pub(crate) attendee: Vec<Attendee>,
    pub(crate) categories: Vec<Categories>,
    pub(crate) comment: Vec<Comment>,
    pub(crate) contact: Vec<Contact>,
    pub(crate) exdate: Vec<ExceptionDateTimes>,
    pub(crate) rstatus: Vec<RequestStatus>,
    pub(crate) related: Vec<RelatedTo>,
    pub(crate) resources: Vec<Resources>,
    pub(crate) rdate: Vec<RecurrenceDateTimes>,
    /// RFC 7986 §5.9/§5.10/§5.11 — core, not feature-gated.
    pub(crate) color: Option<Color>,
    pub(crate) image: Vec<Image>,
    pub(crate) conference: Vec<Conference>,
    pub(crate) xprop: Vec<Xprop>,
    pub(crate) iana: Vec<Iana>,
    pub(crate) alarms: Vec<Alarm>,
}

impl Event {
    /// The `DTSTAMP` property.
    pub fn dtstamp(&self) -> &DateTimeStamp {
        &self.dtstamp
    }

    /// The `UID` property.
    pub fn uid(&self) -> &Uid {
        &self.uid
    }

    /// The `DTSTART` property, if present.
    pub fn dtstart(&self) -> Option<&DateTimeStart> {
        self.dtstart.as_ref()
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

    /// The `GEO` property, if present.
    pub fn geo(&self) -> Option<&Geo> {
        self.geo.as_ref()
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

    /// The `STATUS` property, if present.
    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }

    /// The `SUMMARY` property, if present.
    pub fn summary(&self) -> Option<&Summary> {
        self.summary.as_ref()
    }

    /// The `TRANSP` property, if present.
    pub fn transp(&self) -> Option<&TimeTransparency> {
        self.transp.as_ref()
    }

    /// The `URL` property, if present.
    pub fn url(&self) -> Option<&UniformResourceLocator> {
        self.url.as_ref()
    }

    /// The `RECURRENCE-ID` property, if present.
    pub fn recurid(&self) -> Option<&RecurrenceId> {
        self.recurid.as_ref()
    }

    /// The `RRULE` property, if present.
    pub fn rrule(&self) -> Option<&RRule> {
        self.rrule.as_ref()
    }

    /// The `DTEND` property, if present.
    pub fn dtend(&self) -> Option<&DateTimeEnd> {
        self.dtend.as_ref()
    }

    /// The `DURATION` property, if present.
    pub fn duration(&self) -> Option<&Duration> {
        self.duration.as_ref()
    }

    /// The `ATTACH` properties.
    pub fn attach(&self) -> &[Attachment] {
        &self.attach
    }

    /// The `ATTENDEE` properties.
    pub fn attendee(&self) -> &[Attendee] {
        &self.attendee
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

    /// The `REQUEST-STATUS` properties.
    pub fn rstatus(&self) -> &[RequestStatus] {
        &self.rstatus
    }

    /// The `RELATED-TO` properties.
    pub fn related(&self) -> &[RelatedTo] {
        &self.related
    }

    /// The `RESOURCES` properties.
    pub fn resources(&self) -> &[Resources] {
        &self.resources
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

    /// The `VALARM` sub-components attached to this event.
    pub fn alarms(&self) -> &[Alarm] {
        &self.alarms
    }

    /// The `COLOR` property, if present (RFC 7986 §5.9).
    pub fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    /// The `IMAGE` properties (RFC 7986 §5.10).
    pub fn image(&self) -> &[Image] {
        &self.image
    }

    /// The `CONFERENCE` properties (RFC 7986 §5.11).
    pub fn conference(&self) -> &[Conference] {
        &self.conference
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:VEVENT\r\n")?;
        write!(f, "{}\r\n", self.dtstamp)?;
        write!(f, "{}\r\n", self.uid)?;
        if let Some(v) = &self.dtstart {
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
        if let Some(v) = &self.geo {
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
        if let Some(v) = &self.status {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.summary {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.transp {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.url {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.recurid {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.rrule {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.dtend {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.duration {
            write!(f, "{v}\r\n")?;
        }
        write_lines(f, &self.attach)?;
        write_lines(f, &self.attendee)?;
        write_lines(f, &self.categories)?;
        write_lines(f, &self.comment)?;
        write_lines(f, &self.contact)?;
        write_lines(f, &self.exdate)?;
        write_lines(f, &self.rstatus)?;
        write_lines(f, &self.related)?;
        write_lines(f, &self.resources)?;
        write_lines(f, &self.rdate)?;
        if let Some(v) = &self.color {
            write!(f, "{v}\r\n")?;
        }
        write_lines(f, &self.image)?;
        write_lines(f, &self.conference)?;
        write_lines(f, &self.xprop)?;
        write_lines(f, &self.iana)?;
        write_components(f, &self.alarms)?;
        write!(f, "END:VEVENT\r\n")
    }
}

/// [`EventBuilder`]'s state before either `DTEND` or `DURATION` has been
/// set.
#[derive(Debug)]
pub struct Unset;

/// [`EventBuilder`]'s state once `DTEND` has been set — `.duration()` no
/// longer exists on the builder in this state.
#[derive(Debug)]
pub struct HasDtend;

/// [`EventBuilder`]'s state once `DURATION` has been set — `.dtend()` no
/// longer exists on the builder in this state.
#[derive(Debug)]
pub struct HasDuration;

/// Builder for [`Event`]. Enforces RFC 5545 §3.6.1's `DTEND`/`DURATION`
/// mutual exclusion at compile time via the type-state pattern: `.dtend()`
/// and `.duration()` each move the builder into a distinct state
/// ([`HasDtend`]/[`HasDuration`]), and only one of the two setters is ever
/// callable from [`Unset`] — the other simply doesn't exist on the
/// resulting type, so code that tries to call both doesn't compile. Every
/// other rule RFC 5545 places on `VEVENT` (value-type/`TZID` matching
/// between `DTSTART` and its siblings, the `METHOD`-conditional `DTSTART`
/// requirement) can't be resolved until the actual property values are
/// known, so those stay runtime checks in [`Self::build`], exactly
/// mirroring the parser's own internal builder.
#[derive(Debug)]
pub struct EventBuilder<S = Unset> {
    dtstamp: DateTimeStamp,
    uid: Uid,
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
    color: Option<Color>,
    image: Vec<Image>,
    conference: Vec<Conference>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
    alarms: Vec<Alarm>,
    _state: PhantomData<S>,
}

impl<S> EventBuilder<S> {
    /// Moves every field across into a builder tagged with a different
    /// state, without touching their values — the only thing that changes
    /// is which methods are available on the result.
    fn retag<S2>(self) -> EventBuilder<S2> {
        EventBuilder {
            dtstamp: self.dtstamp,
            uid: self.uid,
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
            color: self.color,
            image: self.image,
            conference: self.conference,
            xprop: self.xprop,
            iana: self.iana,
            alarms: self.alarms,
            _state: PhantomData,
        }
    }
}

impl EventBuilder<Unset> {
    /// Starts building a `VEVENT` from its two properties RFC 5545 §3.6.1
    /// requires unconditionally: `DTSTAMP` and `UID`.
    pub fn new(dtstamp: DateTimeStamp, uid: Uid) -> Self {
        Self {
            dtstamp,
            uid,
            dtstart: None,
            class: None,
            created: None,
            description: None,
            geo: None,
            last_mod: None,
            location: None,
            organizer: None,
            priority: None,
            seq: None,
            status: None,
            summary: None,
            transp: None,
            url: None,
            recurid: None,
            rrule: None,
            dtend: None,
            duration: None,
            attach: Vec::new(),
            attendee: Vec::new(),
            categories: Vec::new(),
            comment: Vec::new(),
            contact: Vec::new(),
            exdate: Vec::new(),
            rstatus: Vec::new(),
            related: Vec::new(),
            resources: Vec::new(),
            rdate: Vec::new(),
            color: None,
            image: Vec::new(),
            conference: Vec::new(),
            xprop: Vec::new(),
            iana: Vec::new(),
            alarms: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Sets `DTEND` (RFC 5545 §3.6.1). Mutually exclusive with
    /// [`Self::duration`] — moves the builder into [`HasDtend`], on which
    /// `.duration()` doesn't exist.
    pub fn dtend(self, dtend: DateTimeEnd) -> EventBuilder<HasDtend> {
        EventBuilder {
            dtend: Some(dtend),
            ..self.retag()
        }
    }

    /// Sets `DURATION` (RFC 5545 §3.6.1). Mutually exclusive with
    /// [`Self::dtend`] — moves the builder into [`HasDuration`], on which
    /// `.dtend()` doesn't exist.
    pub fn duration(self, duration: Duration) -> EventBuilder<HasDuration> {
        EventBuilder {
            duration: Some(duration),
            ..self.retag()
        }
    }
}

impl<S> EventBuilder<S> {
    /// Sets `DTSTART`. REQUIRED unless the enclosing `VCALENDAR` specifies
    /// `METHOD` (RFC 5545 §3.6.1) — see [`Self::build`]'s `has_method`
    /// parameter.
    pub fn dtstart(mut self, v: DateTimeStart) -> Self {
        self.dtstart = Some(v);
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

    /// Sets `GEO`.
    pub fn geo(mut self, v: Geo) -> Self {
        self.geo = Some(v);
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

    /// Sets `STATUS`.
    pub fn status(mut self, v: Status) -> Self {
        self.status = Some(v);
        self
    }

    /// Sets `SUMMARY`.
    pub fn summary(mut self, v: Summary) -> Self {
        self.summary = Some(v);
        self
    }

    /// Sets `TRANSP`.
    pub fn transp(mut self, v: TimeTransparency) -> Self {
        self.transp = Some(v);
        self
    }

    /// Sets `URL`.
    pub fn url(mut self, v: UniformResourceLocator) -> Self {
        self.url = Some(v);
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

    /// Adds an `ATTACH` property.
    pub fn attach(mut self, v: Attachment) -> Self {
        self.attach.push(v);
        self
    }

    /// Adds an `ATTENDEE` property.
    pub fn attendee(mut self, v: Attendee) -> Self {
        self.attendee.push(v);
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

    /// Adds a `REQUEST-STATUS` property.
    pub fn rstatus(mut self, v: RequestStatus) -> Self {
        self.rstatus.push(v);
        self
    }

    /// Adds a `RELATED-TO` property.
    pub fn related(mut self, v: RelatedTo) -> Self {
        self.related.push(v);
        self
    }

    /// Adds a `RESOURCES` property.
    pub fn resources(mut self, v: Resources) -> Self {
        self.resources.push(v);
        self
    }

    /// Adds an `RDATE` property.
    pub fn rdate(mut self, v: RecurrenceDateTimes) -> Self {
        self.rdate.push(v);
        self
    }

    /// Sets `COLOR` (RFC 7986 §5.9).
    pub fn color(mut self, v: Color) -> Self {
        self.color = Some(v);
        self
    }

    /// Adds an `IMAGE` property (RFC 7986 §5.10).
    pub fn image(mut self, v: Image) -> Self {
        self.image.push(v);
        self
    }

    /// Adds a `CONFERENCE` property (RFC 7986 §5.11).
    pub fn conference(mut self, v: Conference) -> Self {
        self.conference.push(v);
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

    /// Adds a `VALARM` sub-component, already built via
    /// [`crate::components::alarm::AlarmBuilder`].
    pub fn alarm(mut self, v: Alarm) -> Self {
        self.alarms.push(v);
        self
    }

    /// Validates the cross-field rules RFC 5545 §3.6.1 places on `VEVENT`
    /// and assembles the finished [`Event`]. `has_method` is whether the
    /// enclosing `VCALENDAR` specifies a `METHOD` property — `DTSTART` is
    /// only REQUIRED here when it doesn't; pass `true` if this `Event`
    /// will only ever be used inside a `VCALENDAR` that sets `METHOD`.
    pub fn build(self, has_method: bool) -> Result<Event, ComponentError> {
        if !has_method && self.dtstart.is_none() {
            return Err(ComponentError::MissingField("DTSTART"));
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

        Ok(Event {
            dtstamp: self.dtstamp,
            uid: self.uid,
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
            color: self.color,
            image: self.image,
            conference: self.conference,
            xprop: self.xprop,
            iana: self.iana,
            alarms: self.alarms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        properties::{DateTimeEndBuilder, DateTimeStartBuilder},
        values::{DateOrDatetime, DateTime},
    };
    use chrono::{TimeZone, Utc};

    fn dtstamp() -> DateTimeStamp {
        DateTimeStamp::new(DateTime::Utc(
            Utc.with_ymd_and_hms(1997, 9, 1, 13, 0, 0).unwrap(),
        ))
    }

    fn uid() -> Uid {
        Uid::new("123@example.com".into())
    }

    #[test]
    // rustfmt's `format_strings` wrapping of this literal corrupts its
    // runtime value (it splits inside a `\r\n` escape pair, not between
    // whole escapes) — skip it here rather than let a formatting pass
    // silently reintroduce that bug.
    #[rustfmt::skip]
    fn event_builder_round_trips_a_minimal_event() {
        let dtstart = DateTimeStartBuilder::new(DateOrDatetime::DateTime(
            DateTime::Utc(Utc.with_ymd_and_hms(1997, 9, 3, 16, 30, 0).unwrap()),
        ))
        .build();
        let event = EventBuilder::new(dtstamp(), uid())
            .dtstart(dtstart)
            .build(false)
            .unwrap();
        assert_eq!(
            event.to_string(),
            "BEGIN:VEVENT\r\nDTSTAMP:19970901T130000Z\r\nUID:123@example.com\r\nDTSTART:19970903T163000Z\r\nEND:VEVENT\r\n"
        );
    }

    #[test]
    fn event_builder_requires_dtstart_unless_has_method() {
        assert!(matches!(
            EventBuilder::new(dtstamp(), uid()).build(false),
            Err(ComponentError::MissingField("DTSTART"))
        ));
        assert!(EventBuilder::new(dtstamp(), uid()).build(true).is_ok());
    }

    #[test]
    fn event_builder_dtend_rejects_a_mismatched_value_type() {
        let dtstart = DateTimeStartBuilder::new(DateOrDatetime::DateTime(
            DateTime::Utc(Utc.with_ymd_and_hms(1997, 9, 3, 16, 30, 0).unwrap()),
        ))
        .build();
        let dtend = DateTimeEndBuilder::new(DateOrDatetime::Date(
            crate::values::Date::try_from(b"19970904".as_slice()).unwrap(),
        ))
        .build();
        let result = EventBuilder::new(dtstamp(), uid())
            .dtstart(dtstart)
            .dtend(dtend)
            .build(false);
        assert!(matches!(
            result,
            Err(ComponentError::MismatchedValueType("DTEND", "DTSTART"))
        ));
    }

    #[test]
    fn event_builder_accepts_duration_instead_of_dtend() {
        // `.dtend()` and `.duration()` are mutually exclusive by
        // construction: each moves `EventBuilder` into a state where the
        // other setter doesn't exist, so there's no runtime check to test
        // here — a program calling both simply wouldn't compile.
        let dtstart = DateTimeStartBuilder::new(DateOrDatetime::DateTime(
            DateTime::Utc(Utc.with_ymd_and_hms(1997, 9, 3, 16, 30, 0).unwrap()),
        ))
        .build();
        let event = EventBuilder::new(dtstamp(), uid())
            .dtstart(dtstart)
            .duration(Duration::new(crate::values::Duration::new(
                chrono::Duration::hours(1),
            )))
            .build(false)
            .unwrap();
        assert!(event.dtend().is_none());
        assert!(event.duration().is_some());
    }
}
