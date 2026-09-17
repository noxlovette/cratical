use crate::{
    params::{
        CalendarUserType, CommonName, Delegatees, Delegators,
        DirectoryEntryReference, Language, Member, ParticipationStatus,
        RecurrenceIdentifierRange, RelationshipType, Rsvp, SentBy,
        TimeZoneIdentifier, ValueDataType,
    },
    properties::{
        AltrepLanguageParams, ParameterError, SharedParams, param_name,
        param_segments, param_value,
    },
    values::{CalendarUserAddress, DateOrDatetime, Text, Uri},
};

/// This property defines an "Attendee" within a calendar component.
///
/// Example:
///
/// > ATTENDEE;ROLE=REQ-PARTICIPANT;DELEGATED-FROM="mailto:bob@example.com";
/// > PARTSTAT=ACCEPTED;CN=Jane Doe:mailto:jdoe@example.com
///
/// [Section 3.8.4.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.1)
#[derive(Debug)]
pub struct Attendee {
    value: CalendarUserAddress,
    params: AttendeeParams,
}

impl_try_from_bytes!(Attendee, CalendarUserAddress, AttendeeParams);

/// Builder for [`Attendee`].
#[derive(Debug)]
pub struct AttendeeBuilder {
    value: CalendarUserAddress,
    params: AttendeeParams,
}

impl AttendeeBuilder {
    /// Starts a new builder from the attendee's required calendar user
    /// address.
    pub fn new(value: CalendarUserAddress) -> Self {
        Self {
            value,
            params: AttendeeParams::default(),
        }
    }

    /// Sets the `LANGUAGE` parameter.
    pub fn language(mut self, v: Language) -> Self {
        self.params.language = Some(v);
        self
    }

    /// Sets the `CUTYPE` parameter.
    pub fn calendar_user_type(mut self, v: CalendarUserType) -> Self {
        self.params.calendar_user_type = Some(v);
        self
    }

    /// Sets the `MEMBER` parameter.
    pub fn member(mut self, v: Member) -> Self {
        self.params.member = Some(v);
        self
    }

    /// Sets the `PARTSTAT` parameter.
    pub fn status(mut self, v: ParticipationStatus) -> Self {
        self.params.status = Some(v);
        self
    }

    /// Sets the `RSVP` parameter.
    pub fn rsvp(mut self, v: Rsvp) -> Self {
        self.params.rsvp = Some(v);
        self
    }

    /// Sets the `DELEGATED-TO` parameter.
    pub fn delegatees(mut self, v: Delegatees) -> Self {
        self.params.deletegatee = Some(v);
        self
    }

    /// Sets the `DELEGATED-FROM` parameter.
    pub fn delegators(mut self, v: Delegators) -> Self {
        self.params.delegator = Some(v);
        self
    }

    /// Sets the `SENT-BY` parameter.
    pub fn sent_by(mut self, v: SentBy) -> Self {
        self.params.sent_by = Some(v);
        self
    }

    /// Sets the `CN` parameter.
    pub fn common_name(mut self, v: CommonName) -> Self {
        self.params.common_name = Some(v);
        self
    }

    /// Sets the `DIR` parameter.
    pub fn directory(mut self, v: DirectoryEntryReference) -> Self {
        self.params.directory = Some(v);
        self
    }

    /// Finishes the builder, producing an [`Attendee`].
    pub fn build(self) -> Attendee {
        Attendee {
            value: self.value,
            params: self.params,
        }
    }
}

impl std::fmt::Display for Attendee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ATTENDEE{}:{}", self.params, self.value)
    }
}

impl Attendee {
    /// Returns the attendee's calendar user address.
    pub fn value(&self) -> &CalendarUserAddress {
        &self.value
    }

    /// The `LANGUAGE` parameter, if set.
    pub fn language(&self) -> Option<&Language> {
        self.params.language.as_ref()
    }

    /// The `CUTYPE` parameter, if set.
    pub fn calendar_user_type(&self) -> Option<&CalendarUserType> {
        self.params.calendar_user_type.as_ref()
    }

    /// The `MEMBER` parameter, if set.
    pub fn member(&self) -> Option<&Member> {
        self.params.member.as_ref()
    }

    /// The `PARTSTAT` parameter, if set.
    pub fn status(&self) -> Option<&ParticipationStatus> {
        self.params.status.as_ref()
    }

    /// The `RSVP` parameter, if set.
    pub fn rsvp(&self) -> Option<&Rsvp> {
        self.params.rsvp.as_ref()
    }

    /// The `DELEGATED-TO` parameter, if set.
    pub fn delegatees(&self) -> Option<&Delegatees> {
        self.params.deletegatee.as_ref()
    }

    /// The `DELEGATED-FROM` parameter, if set.
    pub fn delegators(&self) -> Option<&Delegators> {
        self.params.delegator.as_ref()
    }

    /// The `SENT-BY` parameter, if set.
    pub fn sent_by(&self) -> Option<&SentBy> {
        self.params.sent_by.as_ref()
    }

    /// The `CN` parameter, if set.
    pub fn common_name(&self) -> Option<&CommonName> {
        self.params.common_name.as_ref()
    }

    /// The `DIR` parameter, if set.
    pub fn directory(&self) -> Option<&DirectoryEntryReference> {
        self.params.directory.as_ref()
    }
}

impl std::ops::Deref for Attendee {
    type Target = CalendarUserAddress;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Parameter bundle for [`Attendee`].
#[derive(Debug, Default)]
struct AttendeeParams {
    shared: SharedParams,
    language: Option<Language>,
    calendar_user_type: Option<CalendarUserType>,
    member: Option<Member>,
    status: Option<ParticipationStatus>,
    rsvp: Option<Rsvp>,
    deletegatee: Option<Delegatees>,
    delegator: Option<Delegators>,
    sent_by: Option<SentBy>,
    common_name: Option<CommonName>,
    directory: Option<DirectoryEntryReference>,
}

impl TryFrom<&[u8]> for AttendeeParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            let value = || param_value(segment);
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"LANGUAGE" => {
                    params.language = Some(value()?.as_slice().try_into()?)
                }
                b"CUTYPE" => {
                    params.calendar_user_type =
                        Some(value()?.as_slice().try_into()?)
                }
                b"MEMBER" => {
                    params.member = Some(value()?.as_slice().try_into()?)
                }
                b"PARTSTAT" => {
                    params.status = Some(value()?.as_slice().try_into()?)
                }
                b"RSVP" => params.rsvp = Some(value()?.as_slice().try_into()?),
                b"DELEGATED-TO" => {
                    params.deletegatee = Some(value()?.as_slice().try_into()?)
                }
                b"DELEGATED-FROM" => {
                    params.delegator = Some(value()?.as_slice().try_into()?)
                }
                b"SENT-BY" => {
                    params.sent_by = Some(value()?.as_slice().try_into()?)
                }
                b"CN" => {
                    params.common_name = Some(value()?.as_slice().try_into()?)
                }
                b"DIR" => {
                    params.directory = Some(value()?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for AttendeeParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.language {
            write!(f, ";LANGUAGE={v}")?;
        }
        if let Some(v) = &self.calendar_user_type {
            write!(f, ";CUTYPE={v}")?;
        }
        if let Some(v) = &self.member {
            write!(f, ";MEMBER={v}")?;
        }
        if let Some(v) = &self.status {
            write!(f, ";PARTSTAT={v}")?;
        }
        if let Some(v) = &self.rsvp {
            write!(f, ";RSVP={v}")?;
        }
        if let Some(v) = &self.deletegatee {
            write!(f, ";DELEGATED-TO={v}")?;
        }
        if let Some(v) = &self.delegator {
            write!(f, ";DELEGATED-FROM={v}")?;
        }
        if let Some(v) = &self.sent_by {
            write!(f, ";SENT-BY={v}")?;
        }
        if let Some(v) = &self.common_name {
            write!(f, ";CN={v}")?;
        }
        if let Some(v) = &self.directory {
            write!(f, ";DIR={v}")?;
        }
        write!(f, "{}", self.shared)
    }
}

/// This property is used to represent contact information or alternately a
/// reference to contact information associated with the calendar component.
///
/// Example:
///
/// > CONTACT:Jim Dolittle\, ABC Industries\, +1-919-555-1234
///
/// [Section 3.8.4.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.2)
#[derive(Debug)]
pub struct Contact {
    value: Text,
    params: AltrepLanguageParams,
}

impl_try_from_bytes!(Contact, Text, AltrepLanguageParams);
impl_altrep_language_builder!(ContactBuilder, Contact, Text);
impl_value_accessor!(Contact, Text);

impl std::fmt::Display for Contact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CONTACT{}:{}", self.params, self.value)
    }
}

/// This property defines the organizer for a calendar component.
///
/// Example:
///
/// > ORGANIZER;CN=John Smith:mailto:jsmith@example.com
///
/// [Section 3.8.4.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.3)
#[derive(Debug)]
pub struct Organizer {
    value: CalendarUserAddress,
    params: OrgParams,
}

impl_try_from_bytes!(Organizer, CalendarUserAddress, OrgParams);

/// Builder for [`Organizer`].
#[derive(Debug)]
pub struct OrganizerBuilder {
    value: CalendarUserAddress,
    params: OrgParams,
}

impl OrganizerBuilder {
    /// Starts a new builder from the organizer's required calendar user
    /// address.
    pub fn new(value: CalendarUserAddress) -> Self {
        Self {
            value,
            params: OrgParams::default(),
        }
    }

    /// Sets the `LANGUAGE` parameter.
    pub fn language(mut self, v: Language) -> Self {
        self.params.language = Some(v);
        self
    }

    /// Sets the `CN` parameter.
    pub fn common_name(mut self, v: CommonName) -> Self {
        self.params.common_name = Some(v);
        self
    }

    /// Sets the `DIR` parameter.
    pub fn directory(mut self, v: DirectoryEntryReference) -> Self {
        self.params.directory = Some(v);
        self
    }

    /// Sets the `SENT-BY` parameter.
    pub fn sent_by(mut self, v: SentBy) -> Self {
        self.params.sent_by = Some(v);
        self
    }

    /// Finishes the builder, producing an [`Organizer`].
    pub fn build(self) -> Organizer {
        Organizer {
            value: self.value,
            params: self.params,
        }
    }
}

impl std::fmt::Display for Organizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ORGANIZER{}:{}", self.params, self.value)
    }
}

impl Organizer {
    /// Returns the organizer's calendar user address.
    pub fn value(&self) -> &CalendarUserAddress {
        &self.value
    }

    /// The `LANGUAGE` parameter, if set.
    pub fn language(&self) -> Option<&Language> {
        self.params.language.as_ref()
    }

    /// The `CN` parameter, if set.
    pub fn common_name(&self) -> Option<&CommonName> {
        self.params.common_name.as_ref()
    }

    /// The `DIR` parameter, if set.
    pub fn directory(&self) -> Option<&DirectoryEntryReference> {
        self.params.directory.as_ref()
    }

    /// The `SENT-BY` parameter, if set.
    pub fn sent_by(&self) -> Option<&SentBy> {
        self.params.sent_by.as_ref()
    }
}

impl std::ops::Deref for Organizer {
    type Target = CalendarUserAddress;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Parameter bundle for [`Organizer`].
#[derive(Debug, Default)]
pub struct OrgParams {
    shared: SharedParams,
    language: Option<Language>,
    common_name: Option<CommonName>,
    directory: Option<DirectoryEntryReference>,
    sent_by: Option<SentBy>,
}

impl TryFrom<&[u8]> for OrgParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            let value = || param_value(segment);
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"LANGUAGE" => {
                    params.language = Some(value()?.as_slice().try_into()?)
                }
                b"CN" => {
                    params.common_name = Some(value()?.as_slice().try_into()?)
                }
                b"DIR" => {
                    params.directory = Some(value()?.as_slice().try_into()?)
                }
                b"SENT-BY" => {
                    params.sent_by = Some(value()?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for OrgParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.language {
            write!(f, ";LANGUAGE={v}")?;
        }
        if let Some(v) = &self.common_name {
            write!(f, ";CN={v}")?;
        }
        if let Some(v) = &self.directory {
            write!(f, ";DIR={v}")?;
        }
        if let Some(v) = &self.sent_by {
            write!(f, ";SENT-BY={v}")?;
        }
        write!(f, "{}", self.shared)
    }
}

/// This property is used in conjunction with the "UID" and "SEQUENCE"
/// property to identify a particular instance of a recurring event, to-do,
/// or journal.
///
/// Example:
///
/// > RECURRENCE-ID;VALUE=DATE:19960401
///
/// [Section 3.8.4.4](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.4)
#[derive(Debug)]
pub struct RecurrenceId {
    value: DateOrDatetime,
    params: RecurrenceParams,
}

impl TryFrom<&[u8]> for RecurrenceId {
    type Error = crate::ast::parser::ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let colon = crate::properties::value_start(v)?;
        let params = RecurrenceParams::try_from(&v[..colon])?;
        let value = DateOrDatetime::try_from(&v[colon + 1..])?
            .resolve_tzid(params.tzid.as_ref());
        Ok(Self { value, params })
    }
}

/// Builder for [`RecurrenceId`].
#[derive(Debug)]
pub struct RecurrenceIdBuilder {
    value: DateOrDatetime,
    tzid: Option<TimeZoneIdentifier>,
    recurrence: Option<RecurrenceIdentifierRange>,
}

impl RecurrenceIdBuilder {
    /// Starts a new builder from the property's required value.
    pub fn new(value: DateOrDatetime) -> Self {
        Self {
            value,
            tzid: None,
            recurrence: None,
        }
    }

    /// Sets the `TZID` parameter, resolving a floating `DATE-TIME` value
    /// against it (RFC 5545 §3.3.5). Has no effect on a `DATE` value.
    pub fn tzid(mut self, tzid: TimeZoneIdentifier) -> Self {
        self.tzid = Some(tzid);
        self
    }

    /// Sets the `RANGE` parameter.
    pub fn range(mut self, range: RecurrenceIdentifierRange) -> Self {
        self.recurrence = Some(range);
        self
    }

    /// Finishes the builder, producing a [`RecurrenceId`].
    pub fn build(self) -> RecurrenceId {
        let value = self.value.resolve_tzid(self.tzid.as_ref());
        let data_type = matches!(value, DateOrDatetime::Date(_))
            .then_some(ValueDataType::Date);
        RecurrenceId {
            value,
            params: RecurrenceParams {
                shared: SharedParams::default(),
                data_type,
                tzid: self.tzid,
                recurrence: self.recurrence,
            },
        }
    }
}

impl RecurrenceId {
    /// The parsed `RECURRENCE-ID` value. Also used internally by the
    /// calendar-wide check that flags two components sharing the same
    /// `UID` and `RECURRENCE-ID` (RFC 5545 §3.8.4.4).
    pub fn value(&self) -> &DateOrDatetime {
        &self.value
    }
}

impl std::fmt::Display for RecurrenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RECURRENCE-ID{}:{}", self.params, self.value)
    }
}

/// Parameter bundle for [`RecurrenceId`].
#[derive(Debug, Default)]
struct RecurrenceParams {
    shared: SharedParams,
    data_type: Option<ValueDataType>,
    tzid: Option<TimeZoneIdentifier>,
    recurrence: Option<RecurrenceIdentifierRange>,
}

impl TryFrom<&[u8]> for RecurrenceParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            let value = || param_value(segment);
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"VALUE" => {
                    params.data_type = Some(value()?.as_slice().try_into()?)
                }
                b"TZID" => params.tzid = Some(value()?.as_slice().try_into()?),
                b"RANGE" => {
                    params.recurrence = Some(value()?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for RecurrenceParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.data_type {
            write!(f, ";VALUE={v}")?;
        }
        if let Some(v) = &self.tzid {
            write!(f, ";TZID={v}")?;
        }
        if let Some(v) = &self.recurrence {
            write!(f, ";RANGE={v}")?;
        }
        write!(f, "{}", self.shared)
    }
}

/// This property is used to represent a relationship or reference between
/// one calendar component and another.  The property value consists of the
/// persistent, globally unique identifier of another calendar component.
///
/// Example:
///
/// > RELATED-TO:jsmith.part7.19960817T083000.xyzMail@example.com
///
/// [Section 3.8.4.5](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.5)
#[derive(Debug)]
pub struct RelatedTo {
    // Per this crate's convention (see the Uid property just above),
    // properties may only carry a value.rs type, never another property.
    // RELATED-TO's value type is UID, i.e. plain TEXT.
    value: Text,
    params: RelatedToParams,
}

impl_try_from_bytes!(RelatedTo, Text, RelatedToParams);

impl RelatedTo {
    /// Constructs a new `RELATED-TO` property from its value and an
    /// optional `RELTYPE` parameter.
    pub fn new(value: Text, rt: Option<RelationshipType>) -> Self {
        Self {
            value,
            params: RelatedToParams {
                shared: SharedParams::default(),
                rt,
            },
        }
    }
}

impl std::fmt::Display for RelatedTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RELATED-TO{}:{}", self.params, self.value)
    }
}

/// Parameter bundle for [`RelatedTo`].
#[derive(Debug, Default)]
struct RelatedToParams {
    shared: SharedParams,
    rt: Option<RelationshipType>,
}

impl TryFrom<&[u8]> for RelatedToParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"RELTYPE" => {
                    params.rt =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for RelatedToParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.rt {
            write!(f, ";RELTYPE={v}")?;
        }
        write!(f, "{}", self.shared)
    }
}

/// This property defines a Uniform Resource Locator (URL) associated with
/// the iCalendar object.
///
/// Example:
///
/// > URL:http://example.com/pub/busy/jpublic-01.ifb
///
/// [Section 3.8.4.6](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.6)
#[derive(Debug)]
pub struct UniformResourceLocator {
    value: Uri,
    params: SharedParams,
}

impl_try_from_bytes!(UniformResourceLocator, Uri);
impl_simple_property!(UniformResourceLocator, Uri);

impl std::fmt::Display for UniformResourceLocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "URL{}:{}", self.params, self.value)
    }
}

/// This property defines the persistent, globally unique identifier for the
/// calendar component.  The UID itself MUST be a globally unique identifier.
/// The generator of the identifier MUST guarantee that the identifier is
/// unique.  There are several algorithms that can be used to accomplish
/// this.  The identifier is recommended to be the identical syntax to the
/// [RFC5322] `Message-ID` header field.  In this case, the identifier would
/// be an email message identifier prepended with the "UID:" label.
///
/// Example:
///
/// > UID:19960401T080045Z-4000F192713-0052@example.com
///
/// [Section 3.8.4.7](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.4.7)
#[derive(Debug)]
pub struct Uid {
    value: Text,
    params: SharedParams,
}

impl_try_from_bytes!(Uid);
impl_simple_property!(Uid, Text);

impl Uid {
    /// The `UID` text — used by the calendar-wide check that flags two
    /// components sharing the same `UID` and `RECURRENCE-ID`.
    pub(crate) fn as_str(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UID{}:{}", self.params, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attendee_cn_decodes_rfc_6868_caret_sequences() {
        // collective-icalendar/calendars/rfc_6868.ics
        let attendee = Attendee::try_from(
            b";CN=George Herman ^'Babe^' Ruth:mailto:babe@example.com"
                .as_slice(),
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", attendee.params.common_name.unwrap()),
            r#"CommonName(Text("George Herman \"Babe\" Ruth"))"#
        );
    }

    #[test]
    fn attendee_cn_still_accepts_quoted_form() {
        let attendee = Attendee::try_from(
            br#";CN="Jane Doe":mailto:jdoe@example.com"#.as_slice(),
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", attendee.params.common_name.unwrap()),
            r#"CommonName(Text("Jane Doe"))"#
        );
    }

    #[test]
    fn related_to_reltype_falls_back_to_iana_for_rfc_9253_values() {
        // collective-icalendar/calendars/rfc_9253_related_to.ics, unfolded
        // per RFC 5545 §3.1 — RFC 9253 §5.1 adds RELTYPE values
        // (STARTTOSTART/STARTTOFINISH/etc.) that RelationshipType doesn't
        // model as a dedicated variant, so they must round-trip through its
        // own X/Iana fallback arm rather than erroring.
        let plain = RelatedTo::try_from(
            b":jsmith.part7.19960817T083000.xyzMail@example.com".as_slice(),
        )
        .unwrap();
        assert!(plain.params.rt.is_none());
        assert_eq!(
            plain.value.as_str(),
            "jsmith.part7.19960817T083000.xyzMail@example.com"
        );

        let by_uid = RelatedTo::try_from(
            b";VALUE=UID:19960401-080045-4000F192713-0052@example.com"
                .as_slice(),
        )
        .unwrap();
        // VALUE isn't modeled by RelatedToParams either (only RELTYPE is) —
        // it falls to the same shared passthrough bucket as any other
        // unrecognized param.
        assert!(by_uid.params.rt.is_none());
        assert_eq!(by_uid.params.shared.iana[0].as_str(), "VALUE=UID");
        assert_eq!(
            by_uid.value.as_str(),
            "19960401-080045-4000F192713-0052@example.com"
        );

        let start_to_finish = RelatedTo::try_from(
            b";VALUE=URI;RELTYPE=STARTTOFINISH:https://example.com/caldav/user/jb/cal/19960401-080045-4000F192713.ics"
                .as_slice(),
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", start_to_finish.params.rt),
            r#"Some(Iana(Text("STARTTOFINISH")))"#
        );
        assert_eq!(start_to_finish.params.shared.iana[0].as_str(), "VALUE=URI");
        assert_eq!(
            start_to_finish.value.as_str(),
            "https://example.com/caldav/user/jb/cal/19960401-080045-4000F192713.ics"
        );
    }

    #[test]
    fn related_to_gap_param_falls_back_to_shared_passthrough() {
        // collective-icalendar/calendars/rfc_9253_gap.ics — RFC 9253 §6's
        // GAP parameter isn't a recognized RelatedTo param name at all, so
        // it must land in SharedParams's x-param/iana passthrough bucket
        // rather than erroring, alongside RELTYPE=STARTTOSTART falling back
        // the same way as the sibling test above.
        let related = RelatedTo::try_from(
            b";VALUE=UID;RELTYPE=STARTTOSTART;GAP=P1W:1".as_slice(),
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", related.params.rt),
            r#"Some(Iana(Text("STARTTOSTART")))"#
        );
        assert_eq!(related.params.shared.iana[0].as_str(), "VALUE=UID");
        assert_eq!(related.params.shared.iana[1].as_str(), "GAP=P1W");
        assert_eq!(related.value.as_str(), "1");
    }

    #[test]
    fn attendee_delegation_params_parse_multi_value_quoted_lists() {
        // collective-icalendar/calendars/rfc_7256_multi_value_parameters.ics,
        // the `UID:list` VEVENT — DELEGATED-TO/DELEGATED-FROM/MEMBER (RFC
        // 7256 / RFC 5545 §3.2) each carry two comma-separated, individually
        // DQUOTE-quoted CAL-ADDRESS values in a single parameter.
        let delegated_to = Attendee::try_from(
            br#";DELEGATED-TO="mailto:jdoe@example.com","mailto:jqpublic@example.com":mailto:jsmith@example.com"#
                .as_slice(),
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", delegated_to.params.deletegatee),
            r#"Some(Delegatees([CalendarUserAddress(Uri(Url { scheme: "mailto", cannot_be_a_base: true, username: "", password: None, host: None, port: None, path: "jdoe@example.com", query: None, fragment: None })), CalendarUserAddress(Uri(Url { scheme: "mailto", cannot_be_a_base: true, username: "", password: None, host: None, port: None, path: "jqpublic@example.com", query: None, fragment: None }))]))"#
        );

        let delegated_from = Attendee::try_from(
            br#";DELEGATED-FROM="mailto:jsmith@example.com","mailto:jdoe@example.com":mailto:jdoe@example.com"#
                .as_slice(),
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", delegated_from.params.delegator),
            r#"Some(Delegators([CalendarUserAddress(Uri(Url { scheme: "mailto", cannot_be_a_base: true, username: "", password: None, host: None, port: None, path: "jsmith@example.com", query: None, fragment: None })), CalendarUserAddress(Uri(Url { scheme: "mailto", cannot_be_a_base: true, username: "", password: None, host: None, port: None, path: "jdoe@example.com", query: None, fragment: None }))]))"#
        );

        let member = Attendee::try_from(
            br#";MEMBER="mailto:projectA@example.com","mailto:projectB@example.com":mailto:janedoe@example.com"#
                .as_slice(),
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", member.params.member),
            r#"Some(Member([CalendarUserAddress(Uri(Url { scheme: "mailto", cannot_be_a_base: true, username: "", password: None, host: None, port: None, path: "projectA@example.com", query: None, fragment: None })), CalendarUserAddress(Uri(Url { scheme: "mailto", cannot_be_a_base: true, username: "", password: None, host: None, port: None, path: "projectB@example.com", query: None, fragment: None }))]))"#
        );
    }

    #[test]
    fn attendee_delegation_params_parse_single_value_quoted_lists() {
        // collective-icalendar/calendars/rfc_7256_multi_value_parameters.ics,
        // the `UID:single` VEVENT — same three params, but with only one
        // quoted value each, confirming the comma-list grammar degrades
        // cleanly to a one-element list rather than requiring 2+ values.
        let delegated_to = Attendee::try_from(
            br#";DELEGATED-TO="mailto:jdoe@example.com":mailto:jsmith@example.com"#
                .as_slice(),
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", delegated_to.params.deletegatee),
            r#"Some(Delegatees([CalendarUserAddress(Uri(Url { scheme: "mailto", cannot_be_a_base: true, username: "", password: None, host: None, port: None, path: "jdoe@example.com", query: None, fragment: None }))]))"#
        );

        let delegated_from = Attendee::try_from(
            br#";DELEGATED-FROM="mailto:jsmith@example.com":mailto:jdoe@example.com"#
                .as_slice(),
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", delegated_from.params.delegator),
            r#"Some(Delegators([CalendarUserAddress(Uri(Url { scheme: "mailto", cannot_be_a_base: true, username: "", password: None, host: None, port: None, path: "jsmith@example.com", query: None, fragment: None }))]))"#
        );

        let member = Attendee::try_from(
            br#";MEMBER="mailto:projectA@example.com":mailto:janedoe@example.com"#
                .as_slice(),
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", member.params.member),
            r#"Some(Member([CalendarUserAddress(Uri(Url { scheme: "mailto", cannot_be_a_base: true, username: "", password: None, host: None, port: None, path: "projectA@example.com", query: None, fragment: None }))]))"#
        );
    }

    #[test]
    fn attendee_display_round_trips_the_content_line() {
        let attendee = Attendee::try_from(
            b";ROLE=REQ-PARTICIPANT;PARTSTAT=ACCEPTED;CN=Jane Doe:mailto:jdoe@example.com"
                .as_slice(),
        )
        .unwrap();
        assert_eq!(
            attendee.to_string(),
            "ATTENDEE;PARTSTAT=ACCEPTED;CN=Jane \
             Doe;ROLE=REQ-PARTICIPANT:mailto:jdoe@example.com"
        );
    }

    #[test]
    fn organizer_display_round_trips_the_content_line() {
        let organizer = Organizer::try_from(
            b";CN=John Smith:mailto:jsmith@example.com".as_slice(),
        )
        .unwrap();
        assert_eq!(
            organizer.to_string(),
            "ORGANIZER;CN=John Smith:mailto:jsmith@example.com"
        );
    }

    fn cal_address(s: &str) -> CalendarUserAddress {
        CalendarUserAddress::new(Uri::parse(s).unwrap()).unwrap()
    }

    #[test]
    fn attendee_builder_round_trips_the_content_line() {
        let attendee =
            AttendeeBuilder::new(cal_address("mailto:jdoe@example.com"))
                .status(crate::params::ParticipationStatus::Event(
                    crate::params::PartStatEvent::Accepted,
                ))
                .common_name(crate::params::CommonName::new("Jane Doe".into()))
                .build();
        assert_eq!(
            attendee.to_string(),
            "ATTENDEE;PARTSTAT=ACCEPTED;CN=Jane Doe:mailto:jdoe@example.com"
        );
    }

    #[test]
    fn organizer_builder_round_trips_the_content_line() {
        let organizer =
            OrganizerBuilder::new(cal_address("mailto:jsmith@example.com"))
                .common_name(crate::params::CommonName::new(
                    "John Smith".into(),
                ))
                .build();
        assert_eq!(
            organizer.to_string(),
            "ORGANIZER;CN=John Smith:mailto:jsmith@example.com"
        );
    }

    #[test]
    fn attendee_value_and_param_accessors_read_back_the_parsed_data() {
        let attendee =
            AttendeeBuilder::new(cal_address("mailto:jdoe@example.com"))
                .status(crate::params::ParticipationStatus::Event(
                    crate::params::PartStatEvent::Accepted,
                ))
                .common_name(crate::params::CommonName::new("Jane Doe".into()))
                .rsvp(crate::params::Rsvp::new(true))
                .build();
        assert_eq!(attendee.value().to_string(), "mailto:jdoe@example.com");
        // Deref lets the CalendarUserAddress be reached directly too.
        assert_eq!((*attendee).to_string(), "mailto:jdoe@example.com");
        assert!(matches!(
            attendee.status(),
            Some(crate::params::ParticipationStatus::Event(
                crate::params::PartStatEvent::Accepted
            ))
        ));
        assert_eq!(attendee.common_name().unwrap().to_string(), "Jane Doe");
        assert!(attendee.rsvp().is_some());
        assert!(attendee.language().is_none());
    }

    #[test]
    fn organizer_value_and_param_accessors_read_back_the_parsed_data() {
        let organizer =
            OrganizerBuilder::new(cal_address("mailto:jsmith@example.com"))
                .common_name(crate::params::CommonName::new(
                    "John Smith".into(),
                ))
                .build();
        assert_eq!(organizer.value().to_string(), "mailto:jsmith@example.com");
        assert_eq!((*organizer).to_string(), "mailto:jsmith@example.com");
        assert_eq!(organizer.common_name().unwrap().to_string(), "John Smith");
        assert!(organizer.sent_by().is_none());
    }

    #[test]
    fn recurrence_id_builder_sets_value_date_and_range() {
        let date =
            crate::values::Date::try_from(b"19960401".as_slice()).unwrap();
        let recurrence_id =
            RecurrenceIdBuilder::new(DateOrDatetime::Date(date))
                .range(crate::params::RecurrenceIdentifierRange::ThisAndFuture)
                .build();
        assert_eq!(
            recurrence_id.to_string(),
            "RECURRENCE-ID;VALUE=DATE;RANGE=THISANDFUTURE:19960401"
        );
    }

    #[test]
    fn recurrence_id_value_accessor_reads_back_the_parsed_value() {
        let date =
            crate::values::Date::try_from(b"19960401".as_slice()).unwrap();
        let recurrence_id =
            RecurrenceIdBuilder::new(DateOrDatetime::Date(date)).build();
        assert_eq!(recurrence_id.value(), &DateOrDatetime::Date(date));
    }

    #[test]
    fn contact_builder_round_trips() {
        let contact = ContactBuilder::new(
            "Jim Dolittle, ABC Industries, +1-919-555-1234".into(),
        )
        .build();
        assert_eq!(
            contact.to_string(),
            "CONTACT:Jim Dolittle\\, ABC Industries\\, +1-919-555-1234"
        );
    }

    #[test]
    fn related_to_new_matches_the_parsed_equivalent() {
        let related = RelatedTo::new(
            "jsmith.part7.19960817T083000.xyzMail@example.com".into(),
            None,
        );
        assert_eq!(
            related.to_string(),
            "RELATED-TO:jsmith.part7.19960817T083000.xyzMail@example.com"
        );
    }

    #[test]
    fn url_and_uid_new_match_the_parsed_equivalent() {
        assert_eq!(
            UniformResourceLocator::new(
                Uri::parse("http://example.com/pub/busy/jpublic-01.ifb")
                    .unwrap()
            )
            .to_string(),
            "URL:http://example.com/pub/busy/jpublic-01.ifb"
        );
        assert_eq!(
            Uid::new("19960401T080045Z-4000F192713-0052@example.com".into())
                .to_string(),
            "UID:19960401T080045Z-4000F192713-0052@example.com"
        );
    }
}
