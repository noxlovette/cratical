use crate::properties::{
    Attendee, Comment, Contact, DateTimeEnd, DateTimeStamp, DateTimeStart,
    FreeBusyTime, Iana, Organizer, RequestStatus, Uid, UniformResourceLocator,
    Xprop,
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
