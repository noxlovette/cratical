use std::marker::PhantomData;

use crate::{
    ast::ComponentError,
    components::{alarm::Alarm, write_components, write_lines},
    properties::{
        Attachment, Attendee, Categories, Classification, Color, Comment,
        Completed, Conference, Contact, DateTimeCreated, DateTimeDue,
        DateTimeStamp, DateTimeStart, Description, Duration,
        ExceptionDateTimes, Geo, Iana, Image, LastModified, Location,
        Organizer, PercentComplete, Priority, RRule, RecurrenceDateTimes,
        RecurrenceId, RelatedTo, RequestStatus, Resources, Sequence, Status,
        Summary, Uid, UniformResourceLocator, Xprop,
    },
};

/// A "VTODO" calendar component is a grouping of component
/// properties and possibly "VALARM" calendar components that
/// represent an action-item or assignment.  For example, it can be
/// used to represent an item of work assigned to an individual; such
/// as "turn in travel expense today".
///
/// The "VTODO" calendar component cannot be nested within another
/// calendar component.  However, "VTODO" calendar components can be
/// related to each other or to a "VEVENT" or to a "VJOURNAL" calendar
/// component with the "RELATED-TO" property.
///
/// A "VTODO" calendar component without the "DTSTART" and "DUE" (or
/// "DURATION") properties specifies a to-do that will be associated
/// with each successive calendar date, until it is completed.
///
/// Example:  The following is an example of a "VTODO" calendar
/// component that needs to be completed before May 1st, 2007.  On
/// midnight May 1st, 2007 this to-do would be considered overdue.
///
/// > BEGIN:VTODO
/// >
/// > UID:20070313T123432Z-456553@example.com
/// >
/// > DTSTAMP:20070313T123432Z
/// >
/// > DUE;VALUE=DATE:20070501
/// >
/// > SUMMARY:Submit Quebec Income Tax Return for 2006
/// >
/// > CLASS:CONFIDENTIAL
/// >
/// > CATEGORIES:FAMILY,FINANCE
/// >
/// > STATUS:NEEDS-ACTION
/// >
/// > END:VTODO
/// >
///
/// [Section 3.6.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6.2)
#[derive(Debug)]
pub struct Todo {
    pub(crate) dtstamp: DateTimeStamp,
    pub(crate) uid: Uid,
    pub(crate) class: Option<Classification>,
    pub(crate) completed: Option<Completed>,
    pub(crate) created: Option<DateTimeCreated>,
    pub(crate) description: Option<Description>,
    pub(crate) dtstart: Option<DateTimeStart>,
    pub(crate) geo: Option<Geo>,
    pub(crate) last_mod: Option<LastModified>,
    pub(crate) location: Option<Location>,
    pub(crate) organizer: Option<Organizer>,
    pub(crate) percent: Option<PercentComplete>,
    pub(crate) priority: Option<Priority>,
    pub(crate) recur_id: Option<RecurrenceId>,
    pub(crate) seq: Option<Sequence>,
    pub(crate) status: Option<Status>,
    pub(crate) summary: Option<Summary>,
    pub(crate) url: Option<UniformResourceLocator>,
    pub(crate) rrule: Option<RRule>,
    pub(crate) due: Option<DateTimeDue>,
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

impl Todo {
    /// The `DTSTAMP` property.
    pub fn dtstamp(&self) -> &DateTimeStamp {
        &self.dtstamp
    }

    /// The `UID` property.
    pub fn uid(&self) -> &Uid {
        &self.uid
    }

    /// The `CLASS` property, if present.
    pub fn class(&self) -> Option<&Classification> {
        self.class.as_ref()
    }

    /// The `COMPLETED` property, if present.
    pub fn completed(&self) -> Option<&Completed> {
        self.completed.as_ref()
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

    /// The `PERCENT-COMPLETE` property, if present.
    pub fn percent(&self) -> Option<&PercentComplete> {
        self.percent.as_ref()
    }

    /// The `PRIORITY` property, if present.
    pub fn priority(&self) -> Option<&Priority> {
        self.priority.as_ref()
    }

    /// The `RECURRENCE-ID` property, if present.
    pub fn recur_id(&self) -> Option<&RecurrenceId> {
        self.recur_id.as_ref()
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

    /// The `URL` property, if present.
    pub fn url(&self) -> Option<&UniformResourceLocator> {
        self.url.as_ref()
    }

    /// The `RRULE` property, if present.
    pub fn rrule(&self) -> Option<&RRule> {
        self.rrule.as_ref()
    }

    /// The `DUE` property, if present.
    pub fn due(&self) -> Option<&DateTimeDue> {
        self.due.as_ref()
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

    /// The `VALARM` sub-components attached to this to-do.
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

impl std::fmt::Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:VTODO\r\n")?;
        write!(f, "{}\r\n", self.dtstamp)?;
        write!(f, "{}\r\n", self.uid)?;
        if let Some(v) = &self.class {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.completed {
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
        if let Some(v) = &self.percent {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.priority {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.recur_id {
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
        if let Some(v) = &self.url {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.rrule {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.due {
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
        write!(f, "END:VTODO\r\n")
    }
}

/// [`TodoBuilder`]'s state before either `DUE` or `DURATION` has been set.
#[derive(Debug)]
pub struct Unset;

/// [`TodoBuilder`]'s state once `DUE` has been set — `.duration()` no
/// longer exists on the builder in this state.
#[derive(Debug)]
pub struct HasDue;

/// [`TodoBuilder`]'s state once `DURATION` has been set — `.due()` no
/// longer exists on the builder in this state.
#[derive(Debug)]
pub struct HasDuration;

/// Builder for [`Todo`]. Enforces RFC 5545 §3.6.2's `DUE`/`DURATION`
/// mutual exclusion at compile time via the type-state pattern — see
/// [`crate::components::event::EventBuilder`]'s doc comment for how the
/// state machine works; it's the same shape here. `DURATION` additionally
/// requires `DTSTART` to also be present, and the value-type/`TZID`
/// matching between `DTSTART` and its siblings can't be resolved until the
/// actual property values are known, so both stay runtime checks in
/// [`Self::build`].
#[derive(Debug)]
pub struct TodoBuilder<S = Unset> {
    dtstamp: DateTimeStamp,
    uid: Uid,
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
    color: Option<Color>,
    image: Vec<Image>,
    conference: Vec<Conference>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
    alarms: Vec<Alarm>,
    _state: PhantomData<S>,
}

impl<S> TodoBuilder<S> {
    fn retag<S2>(self) -> TodoBuilder<S2> {
        TodoBuilder {
            dtstamp: self.dtstamp,
            uid: self.uid,
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

impl TodoBuilder<Unset> {
    /// Starts building a `VTODO` from its two properties RFC 5545 §3.6.2
    /// requires unconditionally: `DTSTAMP` and `UID`.
    pub fn new(dtstamp: DateTimeStamp, uid: Uid) -> Self {
        Self {
            dtstamp,
            uid,
            class: None,
            completed: None,
            created: None,
            description: None,
            dtstart: None,
            geo: None,
            last_mod: None,
            location: None,
            organizer: None,
            percent: None,
            priority: None,
            recur_id: None,
            seq: None,
            status: None,
            summary: None,
            url: None,
            rrule: None,
            due: None,
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

    /// Sets `DUE` (RFC 5545 §3.6.2). Mutually exclusive with
    /// [`Self::duration`] — moves the builder into [`HasDue`], on which
    /// `.duration()` doesn't exist.
    pub fn due(self, due: DateTimeDue) -> TodoBuilder<HasDue> {
        TodoBuilder {
            due: Some(due),
            ..self.retag()
        }
    }

    /// Sets `DURATION` (RFC 5545 §3.6.2). Mutually exclusive with
    /// [`Self::due`] — moves the builder into [`HasDuration`], on which
    /// `.due()` doesn't exist. RFC 5545 also requires `DTSTART` to be set
    /// whenever `DURATION` is — checked in [`Self::build`], since it can't
    /// be resolved from the type alone (`.dtstart()` is available in every
    /// state, so nothing stops calling `.duration()` first).
    pub fn duration(self, duration: Duration) -> TodoBuilder<HasDuration> {
        TodoBuilder {
            duration: Some(duration),
            ..self.retag()
        }
    }
}

impl<S> TodoBuilder<S> {
    /// Sets `CLASS`.
    pub fn class(mut self, v: Classification) -> Self {
        self.class = Some(v);
        self
    }

    /// Sets `COMPLETED`.
    pub fn completed(mut self, v: Completed) -> Self {
        self.completed = Some(v);
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

    /// Sets `PERCENT-COMPLETE`.
    pub fn percent(mut self, v: PercentComplete) -> Self {
        self.percent = Some(v);
        self
    }

    /// Sets `PRIORITY`.
    pub fn priority(mut self, v: Priority) -> Self {
        self.priority = Some(v);
        self
    }

    /// Sets `RECURRENCE-ID`.
    pub fn recur_id(mut self, v: RecurrenceId) -> Self {
        self.recur_id = Some(v);
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

    /// Sets `URL`.
    pub fn url(mut self, v: UniformResourceLocator) -> Self {
        self.url = Some(v);
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

    /// Adds a `VALARM` sub-component, already built.
    pub fn alarm(mut self, v: Alarm) -> Self {
        self.alarms.push(v);
        self
    }

    /// Validates the cross-field rules RFC 5545 §3.6.2 places on `VTODO`
    /// and assembles the finished [`Todo`].
    pub fn build(self) -> Result<Todo, ComponentError> {
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

        Ok(Todo {
            dtstamp: self.dtstamp,
            uid: self.uid,
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
        properties::DateTimeDueBuilder,
        values::{DateOrDatetime, DateTime},
    };
    use chrono::{TimeZone, Utc};

    fn dtstamp() -> DateTimeStamp {
        DateTimeStamp::new(DateTime::Utc(
            Utc.with_ymd_and_hms(2007, 3, 13, 12, 34, 32).unwrap(),
        ))
    }

    fn uid() -> Uid {
        Uid::new("20070313T123432Z-456553@example.com".into())
    }

    #[test]
    fn todo_builder_round_trips_a_minimal_todo() {
        let due = DateTimeDueBuilder::new(DateOrDatetime::Date(
            crate::values::Date::try_from(b"20070501".as_slice()).unwrap(),
        ))
        .build();
        let todo = TodoBuilder::new(dtstamp(), uid())
            .due(due)
            .summary(
                crate::properties::SummaryBuilder::new(
                    "Submit Quebec Income Tax Return for 2006".into(),
                )
                .build(),
            )
            .build()
            .unwrap();
        assert_eq!(
            todo.to_string(),
            "BEGIN:VTODO\r\nDTSTAMP:20070313T123432Z\r\nUID:\
             20070313T123432Z-456553@example.com\r\nSUMMARY:Submit Quebec \
             Income Tax Return for \
             2006\r\nDUE;VALUE=DATE:20070501\r\nEND:VTODO\r\n"
        );
    }

    #[test]
    fn todo_builder_rejects_duration_without_dtstart() {
        let result = TodoBuilder::new(dtstamp(), uid())
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
    fn todo_builder_accepts_duration_with_dtstart() {
        let dtstart = crate::properties::DateTimeStartBuilder::new(
            DateOrDatetime::DateTime(DateTime::Utc(
                Utc.with_ymd_and_hms(2007, 3, 13, 12, 0, 0).unwrap(),
            )),
        )
        .build();
        let todo = TodoBuilder::new(dtstamp(), uid())
            .dtstart(dtstart)
            .duration(Duration::new(crate::values::Duration::new(
                chrono::Duration::hours(1),
            )))
            .build()
            .unwrap();
        assert!(todo.due().is_none());
        assert!(todo.duration().is_some());
    }
}
