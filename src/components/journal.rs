use crate::{
    components::write_lines,
    properties::{
        Attachment, Attendee, Categories, Classification, Comment, Contact,
        DateTimeCreated, DateTimeStamp, DateTimeStart, Description,
        ExceptionDateTimes, Iana, LastModified, Organizer, RRule,
        RecurrenceDateTimes, RecurrenceId, RelatedTo, RequestStatus, Sequence,
        Status, Summary, Uid, UniformResourceLocator, Xprop,
    },
};

/// A "VJOURNAL" calendar component is a grouping of
/// component properties that represent one or more descriptive text
/// notes associated with a particular calendar date.  The "DTSTART"
/// property is used to specify the calendar date with which the
/// journal entry is associated.  Generally, it will have a DATE value
/// data type, but it can also be used to specify a DATE-TIME value
/// data type.  Examples of a journal entry include a daily record of
/// a legislative body or a journal entry of individual telephone
/// contacts for the day or an ordered list of accomplishments for the
/// day.  The "VJOURNAL" calendar component can also be used to
/// associate a document with a calendar date.
///
/// The "VJOURNAL" calendar component does not take up time on a
/// calendar.  Hence, it does not play a role in free or busy time
/// searches -- it is as though it has a time transparency value of
/// TRANSPARENT.  It is transparent to any such searches.
///
/// The "VJOURNAL" calendar component cannot be nested within another
/// calendar component.  However, "VJOURNAL" calendar components can
/// be related to each other or to a "VEVENT" or to a "VTODO" calendar
/// component, with the "RELATED-TO" property.
///
/// Example:  The following is an example of the "VJOURNAL" calendar
/// component:
///
/// > BEGIN:VJOURNAL
/// >
/// > UID:19970901T130000Z-123405@example.com
/// >
/// > DTSTAMP:19970901T130000Z
/// >
/// > DTSTART;VALUE=DATE:19970317
/// >
/// > SUMMARY:Staff meeting minutes
/// >
/// > DESCRIPTION:1. Staff meeting: Participants include Joe\,
/// > Lisa\, and Bob. Aurora project plans were reviewed.
/// > There is currently no budget reserves for this project.
/// > Lisa will escalate to management. Next meeting on Tuesday.\n
/// > 2. Telephone Conference: ABC Corp. sales representative
/// > called to discuss new printer. Promised to get us a demo by
/// > Friday.\n3. Henry Miller (Handsoff Insurance): Car was
/// > totaled by tree. Is looking into a loaner car. 555-2323
/// > (tel).
/// >
/// > END:VJOURNAL
/// >
///
/// [Section 3.6.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6.3)
#[derive(Debug)]
pub struct Journal {
    pub(crate) dtstamp: DateTimeStamp,
    pub(crate) uid: Uid,
    pub(crate) class: Option<Classification>,
    pub(crate) created: Option<DateTimeCreated>,
    pub(crate) dtstart: Option<DateTimeStart>,
    pub(crate) last_mod: Option<LastModified>,
    pub(crate) organizer: Option<Organizer>,
    pub(crate) recurid: Option<RecurrenceId>,
    pub(crate) seq: Option<Sequence>,
    pub(crate) status: Option<Status>,
    pub(crate) summary: Option<Summary>,
    pub(crate) url: Option<UniformResourceLocator>,
    pub(crate) rrule: Option<RRule>,
    pub(crate) attach: Vec<Attachment>,
    pub(crate) attendee: Vec<Attendee>,
    pub(crate) categories: Vec<Categories>,
    pub(crate) comment: Vec<Comment>,
    pub(crate) contact: Vec<Contact>,
    pub(crate) description: Vec<Description>,
    pub(crate) exdate: Vec<ExceptionDateTimes>,
    pub(crate) related: Vec<RelatedTo>,
    pub(crate) rdate: Vec<RecurrenceDateTimes>,
    pub(crate) rstatus: Vec<RequestStatus>,
    pub(crate) xprop: Vec<Xprop>,
    pub(crate) iana: Vec<Iana>,
}

impl Journal {
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

    /// The `CREATED` property, if present.
    pub fn created(&self) -> Option<&DateTimeCreated> {
        self.created.as_ref()
    }

    /// The `DTSTART` property, if present.
    pub fn dtstart(&self) -> Option<&DateTimeStart> {
        self.dtstart.as_ref()
    }

    /// The `LAST-MODIFIED` property, if present.
    pub fn last_mod(&self) -> Option<&LastModified> {
        self.last_mod.as_ref()
    }

    /// The `ORGANIZER` property, if present.
    pub fn organizer(&self) -> Option<&Organizer> {
        self.organizer.as_ref()
    }

    /// The `RECURRENCE-ID` property, if present.
    pub fn recurid(&self) -> Option<&RecurrenceId> {
        self.recurid.as_ref()
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

    /// The `DESCRIPTION` properties.
    pub fn description(&self) -> &[Description] {
        &self.description
    }

    /// The `EXDATE` properties.
    pub fn exdate(&self) -> &[ExceptionDateTimes] {
        &self.exdate
    }

    /// The `RELATED-TO` properties.
    pub fn related(&self) -> &[RelatedTo] {
        &self.related
    }

    /// The `RDATE` properties.
    pub fn rdate(&self) -> &[RecurrenceDateTimes] {
        &self.rdate
    }

    /// The `REQUEST-STATUS` properties.
    pub fn rstatus(&self) -> &[RequestStatus] {
        &self.rstatus
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

impl std::fmt::Display for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:VJOURNAL\r\n")?;
        write!(f, "{}\r\n", self.dtstamp)?;
        write!(f, "{}\r\n", self.uid)?;
        if let Some(v) = &self.class {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.created {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.dtstart {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.last_mod {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.organizer {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.recurid {
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
        write_lines(f, &self.attach)?;
        write_lines(f, &self.attendee)?;
        write_lines(f, &self.categories)?;
        write_lines(f, &self.comment)?;
        write_lines(f, &self.contact)?;
        write_lines(f, &self.description)?;
        write_lines(f, &self.exdate)?;
        write_lines(f, &self.related)?;
        write_lines(f, &self.rdate)?;
        write_lines(f, &self.rstatus)?;
        write_lines(f, &self.xprop)?;
        write_lines(f, &self.iana)?;
        write!(f, "END:VJOURNAL\r\n")
    }
}
