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
                b"LANGUAGE" => params.language = Some(value()?.as_slice().try_into()?),
                b"CUTYPE" => {
                    params.calendar_user_type = Some(value()?.as_slice().try_into()?)
                }
                b"MEMBER" => params.member = Some(value()?.as_slice().try_into()?),
                b"PARTSTAT" => params.status = Some(value()?.as_slice().try_into()?),
                b"RSVP" => params.rsvp = Some(value()?.as_slice().try_into()?),
                b"DELEGATED-TO" => {
                    params.deletegatee = Some(value()?.as_slice().try_into()?)
                }
                b"DELEGATED-FROM" => {
                    params.delegator = Some(value()?.as_slice().try_into()?)
                }
                b"SENT-BY" => params.sent_by = Some(value()?.as_slice().try_into()?),
                b"CN" => params.common_name = Some(value()?.as_slice().try_into()?),
                b"DIR" => params.directory = Some(value()?.as_slice().try_into()?),
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
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
                b"LANGUAGE" => params.language = Some(value()?.as_slice().try_into()?),
                b"CN" => params.common_name = Some(value()?.as_slice().try_into()?),
                b"DIR" => params.directory = Some(value()?.as_slice().try_into()?),
                b"SENT-BY" => params.sent_by = Some(value()?.as_slice().try_into()?),
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
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

impl RecurrenceId {
    /// The parsed `RECURRENCE-ID` value — used by the calendar-wide check
    /// that flags two components sharing the same `UID` and `RECURRENCE-ID`
    /// (RFC 5545 §3.8.4.4).
    pub(crate) fn value(&self) -> &DateOrDatetime {
        &self.value
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
                b"VALUE" => params.data_type = Some(value()?.as_slice().try_into()?),
                b"TZID" => params.tzid = Some(value()?.as_slice().try_into()?),
                b"RANGE" => params.recurrence = Some(value()?.as_slice().try_into()?),
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
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
                    params.rt = Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
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

impl Uid {
    /// The `UID` text — used by the calendar-wide check that flags two
    /// components sharing the same `UID` and `RECURRENCE-ID`.
    pub(crate) fn as_str(&self) -> &str {
        &self.value
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
}
