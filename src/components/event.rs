use crate::{
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
