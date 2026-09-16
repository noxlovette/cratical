use crate::{
    ast::ComponentError,
    components::write_lines,
    properties::{
        Attendee, Comment, Contact, DateTimeEnd, DateTimeStamp, DateTimeStart,
        FreeBusyTime, Iana, Organizer, RequestStatus, Uid,
        UniformResourceLocator, Xprop,
    },
};

/// A "VFREEBUSY" calendar component is a grouping of
/// component properties that represents either a request for free or
/// busy time information, a reply to a request for free or busy time
/// information, or a published set of busy time information.
///
/// When used to request free/busy time information, the "ATTENDEE"
/// property specifies the calendar users whose free/busy time is
/// being requested; the "ORGANIZER" property specifies the calendar
/// user who is requesting the free/busy time; the "DTSTART" and
/// "DTEND" properties specify the window of time for which the free/
/// busy time is being requested; the "UID" and "DTSTAMP" properties
/// are specified to assist in proper sequencing of multiple free/busy
/// time requests.
///
/// When used to reply to a request for free/busy time, the "ATTENDEE"
/// property specifies the calendar user responding to the free/busy
/// time request; the "ORGANIZER" property specifies the calendar user
/// that originally requested the free/busy time; the "FREEBUSY"
/// property specifies the free/busy time information (if it exists);
/// and the "UID" and "DTSTAMP" properties are specified to assist in
/// proper sequencing of multiple free/busy time replies.
///
/// When used to publish busy time, the "ORGANIZER" property specifies
/// the calendar user associated with the published busy time; the
/// "DTSTART" and "DTEND" properties specify an inclusive time window
/// that surrounds the busy time information; the "FREEBUSY" property
/// specifies the published busy time information; and the "DTSTAMP"
/// property specifies the DATE-TIME that iCalendar object was
/// created.
///
/// The "VFREEBUSY" calendar component cannot be nested within another
/// calendar component.  Multiple "VFREEBUSY" calendar components can
/// be specified within an iCalendar object.  This permits the
/// grouping of free/busy information into logical collections, such
/// as monthly groups of busy time information.
///
/// The "VFREEBUSY" calendar component is intended for use in
/// iCalendar object methods involving requests for free time,
/// requests for busy time, requests for both free and busy, and the
/// associated replies.
///
/// Free/Busy information is represented with the "FREEBUSY" property.
/// This property provides a terse representation of time periods.
/// One or more "FREEBUSY" properties can be specified in the
/// "VFREEBUSY" calendar component.
///
/// When present in a "VFREEBUSY" calendar component, the "DTSTART"
/// and "DTEND" properties SHOULD be specified prior to any "FREEBUSY"
/// properties.
///
/// The recurrence properties ("RRULE", "RDATE", "EXDATE") are not
/// permitted within a "VFREEBUSY" calendar component.  Any recurring
/// events are resolved into their individual busy time periods using
/// the "FREEBUSY" property.
///
/// Example:  The following is an example of a "VFREEBUSY" calendar
/// component used to request free or busy time information:
///
/// > BEGIN:VFREEBUSY
/// >
/// > UID:19970901T082949Z-FA43EF@example.com
/// >
/// > ORGANIZER:mailto:jane_doe@example.com
/// >
/// > ATTENDEE:mailto:john_public@example.com
/// >
/// > DTSTART:19971015T050000Z
/// >
/// > DTEND:19971016T050000Z
/// >
/// > DTSTAMP:19970901T083000Z
/// >
/// > END:VFREEBUSY
/// >
///
/// [Section 3.6.4](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6.4)
#[derive(Debug)]
pub struct FreeBusy {
    pub(crate) dtstamp: Option<DateTimeStamp>,
    pub(crate) uid: Option<Uid>,
    pub(crate) contact: Option<Contact>,
    pub(crate) dtstart: Option<DateTimeStart>,
    pub(crate) dtend: Option<DateTimeEnd>,
    pub(crate) organizer: Option<Organizer>,
    pub(crate) url: Option<UniformResourceLocator>,
    pub(crate) attendee: Vec<Attendee>,
    pub(crate) comment: Vec<Comment>,
    pub(crate) freebusy: Vec<FreeBusyTime>,
    pub(crate) rstatus: Vec<RequestStatus>,
    pub(crate) xprop: Vec<Xprop>,
    pub(crate) iana: Vec<Iana>,
}

impl FreeBusy {
    /// The `DTSTAMP` property, if present.
    pub fn dtstamp(&self) -> Option<&DateTimeStamp> {
        self.dtstamp.as_ref()
    }

    /// The `UID` property, if present.
    pub fn uid(&self) -> Option<&Uid> {
        self.uid.as_ref()
    }

    /// The `CONTACT` property, if present.
    pub fn contact(&self) -> Option<&Contact> {
        self.contact.as_ref()
    }

    /// The `DTSTART` property, if present.
    pub fn dtstart(&self) -> Option<&DateTimeStart> {
        self.dtstart.as_ref()
    }

    /// The `DTEND` property, if present.
    pub fn dtend(&self) -> Option<&DateTimeEnd> {
        self.dtend.as_ref()
    }

    /// The `ORGANIZER` property, if present.
    pub fn organizer(&self) -> Option<&Organizer> {
        self.organizer.as_ref()
    }

    /// The `URL` property, if present.
    pub fn url(&self) -> Option<&UniformResourceLocator> {
        self.url.as_ref()
    }

    /// The `ATTENDEE` properties.
    pub fn attendee(&self) -> &[Attendee] {
        &self.attendee
    }

    /// The `COMMENT` properties.
    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    /// The `FREEBUSY` properties.
    pub fn freebusy(&self) -> &[FreeBusyTime] {
        &self.freebusy
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

impl std::fmt::Display for FreeBusy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:VFREEBUSY\r\n")?;
        if let Some(v) = &self.dtstamp {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.uid {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.contact {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.dtstart {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.dtend {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.organizer {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.url {
            write!(f, "{v}\r\n")?;
        }
        write_lines(f, &self.attendee)?;
        write_lines(f, &self.comment)?;
        write_lines(f, &self.freebusy)?;
        write_lines(f, &self.rstatus)?;
        write_lines(f, &self.xprop)?;
        write_lines(f, &self.iana)?;
        write!(f, "END:VFREEBUSY\r\n")
    }
}

/// Builder for [`FreeBusy`]. RFC 5545 §3.6.4 requires nothing on
/// `VFREEBUSY` unconditionally — every field is optional, so unlike most
/// of this module's other builders, [`Self::new`] takes no arguments. The
/// one cross-field rule (`DTEND`'s value type matching `DTSTART`'s) can't
/// be resolved until the actual property values are known, so it stays a
/// runtime check in [`Self::build`].
#[derive(Debug, Default)]
pub struct FreeBusyBuilder {
    dtstamp: Option<DateTimeStamp>,
    uid: Option<Uid>,
    contact: Option<Contact>,
    dtstart: Option<DateTimeStart>,
    dtend: Option<DateTimeEnd>,
    organizer: Option<Organizer>,
    url: Option<UniformResourceLocator>,
    attendee: Vec<Attendee>,
    comment: Vec<Comment>,
    freebusy: Vec<FreeBusyTime>,
    rstatus: Vec<RequestStatus>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
}

impl FreeBusyBuilder {
    /// Starts building an empty `VFREEBUSY`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `DTSTAMP`.
    pub fn dtstamp(mut self, v: DateTimeStamp) -> Self {
        self.dtstamp = Some(v);
        self
    }

    /// Sets `UID`.
    pub fn uid(mut self, v: Uid) -> Self {
        self.uid = Some(v);
        self
    }

    /// Sets `CONTACT`.
    pub fn contact(mut self, v: Contact) -> Self {
        self.contact = Some(v);
        self
    }

    /// Sets `DTSTART`.
    pub fn dtstart(mut self, v: DateTimeStart) -> Self {
        self.dtstart = Some(v);
        self
    }

    /// Sets `DTEND`.
    pub fn dtend(mut self, v: DateTimeEnd) -> Self {
        self.dtend = Some(v);
        self
    }

    /// Sets `ORGANIZER`.
    pub fn organizer(mut self, v: Organizer) -> Self {
        self.organizer = Some(v);
        self
    }

    /// Sets `URL`.
    pub fn url(mut self, v: UniformResourceLocator) -> Self {
        self.url = Some(v);
        self
    }

    /// Adds an `ATTENDEE` property.
    pub fn attendee(mut self, v: Attendee) -> Self {
        self.attendee.push(v);
        self
    }

    /// Adds a `COMMENT` property.
    pub fn comment(mut self, v: Comment) -> Self {
        self.comment.push(v);
        self
    }

    /// Adds a `FREEBUSY` property.
    pub fn freebusy(mut self, v: FreeBusyTime) -> Self {
        self.freebusy.push(v);
        self
    }

    /// Adds a `REQUEST-STATUS` property.
    pub fn rstatus(mut self, v: RequestStatus) -> Self {
        self.rstatus.push(v);
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

    /// Validates the cross-field rule RFC 5545 §3.6.4 places on
    /// `VFREEBUSY` and assembles the finished [`FreeBusy`].
    pub fn build(self) -> Result<FreeBusy, ComponentError> {
        if let Some(dtstart) = self.dtstart.as_ref() {
            dtstart.cmp_value_type(
                self.dtend.as_ref().map(DateTimeEnd::value),
                "DTEND",
            )?;
        }

        Ok(FreeBusy {
            dtstamp: self.dtstamp,
            uid: self.uid,
            contact: self.contact,
            dtstart: self.dtstart,
            dtend: self.dtend,
            organizer: self.organizer,
            url: self.url,
            attendee: self.attendee,
            comment: self.comment,
            freebusy: self.freebusy,
            rstatus: self.rstatus,
            xprop: self.xprop,
            iana: self.iana,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_busy_builder_round_trips_a_minimal_free_busy() {
        let free_busy = FreeBusyBuilder::new()
            .uid(Uid::new("19970901T082949Z-FA43EF@example.com".into()))
            .build()
            .unwrap();
        assert_eq!(
            free_busy.to_string(),
            "BEGIN:VFREEBUSY\r\nUID:19970901T082949Z-FA43EF@example.com\r\\
             nEND:VFREEBUSY\r\n"
        );
    }

    #[test]
    fn free_busy_builder_rejects_a_mismatched_dtend_value_type() {
        let dtstart = crate::properties::DateTimeStartBuilder::new(
            crate::values::DateOrDatetime::DateTime(
                crate::values::DateTime::Utc(
                    chrono::TimeZone::with_ymd_and_hms(
                        &chrono::Utc,
                        1997,
                        10,
                        15,
                        5,
                        0,
                        0,
                    )
                    .unwrap(),
                ),
            ),
        )
        .build();
        let dtend = crate::properties::DateTimeEndBuilder::new(
            crate::values::DateOrDatetime::Date(
                crate::values::Date::try_from(b"19971016".as_slice()).unwrap(),
            ),
        )
        .build();
        let result =
            FreeBusyBuilder::new().dtstart(dtstart).dtend(dtend).build();
        assert!(matches!(
            result,
            Err(ComponentError::MismatchedValueType("DTEND", "DTSTART"))
        ));
    }
}
