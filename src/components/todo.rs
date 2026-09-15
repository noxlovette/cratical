use crate::{
    components::{alarm::Alarm, write_components, write_lines},
    properties::{
        Attachment, Attendee, Categories, Classification, Comment, Completed,
        Contact, DateTimeCreated, DateTimeDue, DateTimeStamp, DateTimeStart,
        Description, Duration, ExceptionDateTimes, Geo, Iana, LastModified,
        Location, Organizer, PercentComplete, Priority, RRule,
        RecurrenceDateTimes, RecurrenceId, RelatedTo, RequestStatus, Resources,
        Sequence, Status, Summary, Uid, UniformResourceLocator, Xprop,
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
        write_lines(f, &self.xprop)?;
        write_lines(f, &self.iana)?;
        write_components(f, &self.alarms)?;
        write!(f, "END:VTODO\r\n")
    }
}
