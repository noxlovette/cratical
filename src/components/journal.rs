use crate::{
    ast::ComponentError,
    components::write_lines,
    properties::{
        Attachment, Attendee, Categories, Classification, Color, Comment,
        Contact, DateTimeCreated, DateTimeStamp, DateTimeStart, Description,
        ExceptionDateTimes, Iana, Image, LastModified, Organizer, RRule,
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
    /// RFC 7986 §5.9/§5.10 — core, not feature-gated (no `CONFERENCE`
    /// here: RFC 7986 §5.11 only allows it on `VEVENT`/`VTODO`).
    pub(crate) color: Option<Color>,
    pub(crate) image: Vec<Image>,
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

    /// The `COLOR` property, if present (RFC 7986 §5.9).
    pub fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    /// The `IMAGE` properties (RFC 7986 §5.10).
    pub fn image(&self) -> &[Image] {
        &self.image
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
        if let Some(v) = &self.color {
            write!(f, "{v}\r\n")?;
        }
        write_lines(f, &self.image)?;
        write_lines(f, &self.xprop)?;
        write_lines(f, &self.iana)?;
        write!(f, "END:VJOURNAL\r\n")
    }
}

/// Builder for [`Journal`]. RFC 5545 §3.6.3 places no mutual-exclusion or
/// "requires" rules on `VJOURNAL` — the only cross-field checks are the
/// same value-type/`TZID` matching between `DTSTART` and its siblings
/// used everywhere else in this crate, which can't be resolved until the
/// actual property values are known, so they stay runtime checks in
/// [`Self::build`].
#[derive(Debug)]
pub struct JournalBuilder {
    dtstamp: DateTimeStamp,
    uid: Uid,
    class: Option<Classification>,
    created: Option<DateTimeCreated>,
    dtstart: Option<DateTimeStart>,
    last_mod: Option<LastModified>,
    organizer: Option<Organizer>,
    recurid: Option<RecurrenceId>,
    seq: Option<Sequence>,
    status: Option<Status>,
    summary: Option<Summary>,
    url: Option<UniformResourceLocator>,
    rrule: Option<RRule>,
    attach: Vec<Attachment>,
    attendee: Vec<Attendee>,
    categories: Vec<Categories>,
    comment: Vec<Comment>,
    contact: Vec<Contact>,
    description: Vec<Description>,
    exdate: Vec<ExceptionDateTimes>,
    related: Vec<RelatedTo>,
    rdate: Vec<RecurrenceDateTimes>,
    rstatus: Vec<RequestStatus>,
    color: Option<Color>,
    image: Vec<Image>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
}

impl JournalBuilder {
    /// Starts building a `VJOURNAL` from its two properties RFC 5545
    /// §3.6.3 requires unconditionally: `DTSTAMP` and `UID`.
    pub fn new(dtstamp: DateTimeStamp, uid: Uid) -> Self {
        Self {
            dtstamp,
            uid,
            class: None,
            created: None,
            dtstart: None,
            last_mod: None,
            organizer: None,
            recurid: None,
            seq: None,
            status: None,
            summary: None,
            url: None,
            rrule: None,
            attach: Vec::new(),
            attendee: Vec::new(),
            categories: Vec::new(),
            comment: Vec::new(),
            contact: Vec::new(),
            description: Vec::new(),
            exdate: Vec::new(),
            related: Vec::new(),
            rdate: Vec::new(),
            rstatus: Vec::new(),
            color: None,
            image: Vec::new(),
            xprop: Vec::new(),
            iana: Vec::new(),
        }
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

    /// Sets `ORGANIZER`.
    pub fn organizer(mut self, v: Organizer) -> Self {
        self.organizer = Some(v);
        self
    }

    /// Sets `RECURRENCE-ID`.
    pub fn recurid(mut self, v: RecurrenceId) -> Self {
        self.recurid = Some(v);
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

    /// Adds a `DESCRIPTION` property.
    pub fn description(mut self, v: Description) -> Self {
        self.description.push(v);
        self
    }

    /// Adds an `EXDATE` property.
    pub fn exdate(mut self, v: ExceptionDateTimes) -> Self {
        self.exdate.push(v);
        self
    }

    /// Adds a `RELATED-TO` property.
    pub fn related(mut self, v: RelatedTo) -> Self {
        self.related.push(v);
        self
    }

    /// Adds an `RDATE` property.
    pub fn rdate(mut self, v: RecurrenceDateTimes) -> Self {
        self.rdate.push(v);
        self
    }

    /// Adds a `REQUEST-STATUS` property.
    pub fn rstatus(mut self, v: RequestStatus) -> Self {
        self.rstatus.push(v);
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

    /// Validates the cross-field rules RFC 5545 §3.6.3 places on
    /// `VJOURNAL` and assembles the finished [`Journal`].
    pub fn build(self) -> Result<Journal, ComponentError> {
        if let Some(dtstart) = self.dtstart.as_ref() {
            dtstart.cmp_until(self.rrule.as_ref())?;
            dtstart.cmp_exdate(&self.exdate)?;
            dtstart.cmp_rdate(&self.rdate)?;
            dtstart.cmp_exdate_tzid(&self.exdate)?;
            dtstart.cmp_rdate_tzid(&self.rdate)?;
        }

        Ok(Journal {
            dtstamp: self.dtstamp,
            uid: self.uid,
            class: self.class,
            created: self.created,
            dtstart: self.dtstart,
            last_mod: self.last_mod,
            organizer: self.organizer,
            recurid: self.recurid,
            seq: self.seq,
            status: self.status,
            summary: self.summary,
            url: self.url,
            rrule: self.rrule,
            attach: self.attach,
            attendee: self.attendee,
            categories: self.categories,
            comment: self.comment,
            contact: self.contact,
            description: self.description,
            exdate: self.exdate,
            related: self.related,
            rdate: self.rdate,
            rstatus: self.rstatus,
            color: self.color,
            image: self.image,
            xprop: self.xprop,
            iana: self.iana,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::DateTime;
    use chrono::{TimeZone, Utc};

    #[test]
    fn journal_builder_round_trips_a_minimal_journal() {
        let journal = JournalBuilder::new(
            DateTimeStamp::new(DateTime::Utc(
                Utc.with_ymd_and_hms(1997, 9, 1, 13, 0, 0).unwrap(),
            )),
            Uid::new("19970901T130000Z-123405@example.com".into()),
        )
        .build()
        .unwrap();
        assert_eq!(
            journal.to_string(),
            "BEGIN:VJOURNAL\r\nDTSTAMP:19970901T130000Z\r\nUID:\
             19970901T130000Z-123405@example.com\r\nEND:VJOURNAL\r\n"
        );
    }
}
