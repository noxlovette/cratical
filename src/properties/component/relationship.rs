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

impl std::fmt::Display for Attendee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ATTENDEE{}:{}", self.params, self.value)
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

impl std::fmt::Display for Organizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ORGANIZER{}:{}", self.params, self.value)
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

impl RecurrenceId {
    /// The parsed `RECURRENCE-ID` value — used by the calendar-wide check
    /// that flags two components sharing the same `UID` and `RECURRENCE-ID`
    /// (RFC 5545 §3.8.4.4).
    pub(crate) fn value(&self) -> &DateOrDatetime {
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
}
