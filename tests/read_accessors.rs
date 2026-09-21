//! Black-box coverage that everything a client needs to *read* back out of a
//! parsed calendar is reachable from outside the crate — properties, their
//! parameters, and the value types underneath. Each test parses wire text
//! taken from an RFC 5545 example and asserts what RFC 5545 says those bytes
//! mean, so it can only compile and pass through the public API.

use chrono::{TimeZone, Utc};
use cratical::{
    Calendar, Component,
    components::event::Event,
    params::{
        AlarmTriggerRelationship, Altrep, CommonName, DirectoryEntryReference,
        Feature, FeatureValue, Fmttype, ImageDisplay, ImageDisplayValue, Label,
        Language, ParticipationRole, Rsvp, TimeZoneIdentifier,
    },
    values::{
        CalendarUserAddress, DateOrDatetime, DateTimeDuration, DateTimePeriod,
        MediaType, Text, Uri,
    },
};

/// Parses `lines` (each without its CRLF) as the body of one `VEVENT`,
/// after the required `DTSTAMP`/`UID`/`DTSTART`, inside a minimal
/// `VCALENDAR`.
fn calendar(lines: &[&str]) -> Calendar {
    let mut src = String::from(
        "BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\n\
         BEGIN:VEVENT\r\nDTSTAMP:19970610T172345Z\r\n\
         UID:19970610T172345Z-AF23B2@example.com\r\n\
         DTSTART:19970714T170000Z\r\n",
    );
    for l in lines {
        src.push_str(l);
        src.push_str("\r\n");
    }
    src.push_str("END:VEVENT\r\nEND:VCALENDAR\r\n");
    Calendar::parse(src.as_bytes())
        .unwrap_or_else(|e| panic!("should parse: {e}\n---\n{src}"))
}

fn event(calendar: &Calendar) -> &Event {
    let Some(Component::Event(event)) = calendar.components().first() else {
        panic!("expected a VEVENT");
    };
    event
}

/// RFC 5545 §3.8.4.1's `ATTENDEE` example with `CN`, `RSVP` and `ROLE`.
#[test]
fn attendee_exposes_address_cn_rsvp_and_role() {
    let c =
        calendar(&["ATTENDEE;CN=John Smith;RSVP=TRUE;ROLE=REQ-PARTICIPANT:\
         mailto:jsmith@example.com"]);
    let attendee = &event(&c).attendee()[0];

    // The bare address is what a client matches on (e.g. against a user's
    // email), not the whole `mailto:` URI.
    assert_eq!(attendee.value().uri().scheme(), "mailto");
    assert_eq!(attendee.value().uri().path(), "jsmith@example.com");
    assert_eq!(attendee.common_name().unwrap().as_str(), "John Smith");
    assert!(attendee.rsvp().unwrap().value());
    assert!(matches!(
        attendee.role(),
        Some(ParticipationRole::ReqParticipant)
    ));
}

/// RFC 5545 §3.2.17: `RSVP=FALSE` is readable as `false`, distinct from the
/// parameter being absent.
#[test]
fn rsvp_false_reads_as_false_and_absent_reads_as_none() {
    let c = calendar(&[
        "ATTENDEE;RSVP=FALSE:mailto:a@example.com",
        "ATTENDEE:mailto:b@example.com",
    ]);
    let attendees = event(&c).attendee();
    assert!(!attendees[0].rsvp().unwrap().value());
    assert!(attendees[1].rsvp().is_none());
}

/// RFC 5545 §3.2.4, §3.2.5, §3.2.18 and §3.2.11: the calendar-address-list
/// and single-address parameters on `ATTENDEE`, and `DIR`.
#[test]
fn attendee_exposes_delegation_membership_sent_by_and_dir() {
    let c = calendar(&["ATTENDEE;MEMBER=\"mailto:ietf-calsch@example.org\";\
         DELEGATED-TO=\"mailto:jdoe@example.com\",\"mailto:jqpublic@example.com\";\
         DELEGATED-FROM=\"mailto:boss@example.com\";\
         SENT-BY=\"mailto:sray@example.com\";\
         DIR=\"ldap://example.com:6666/o=ABC%20Industries,c=US???(cn=Jim%20Dolittle)\":\
         mailto:jsmith@example.com"]);
    let attendee = &event(&c).attendee()[0];

    let paths = |list: &[CalendarUserAddress]| -> Vec<String> {
        list.iter().map(|a| a.uri().path().to_string()).collect()
    };
    assert_eq!(
        paths(attendee.member().unwrap().value()),
        ["ietf-calsch@example.org"]
    );
    assert_eq!(
        paths(attendee.delegatees().unwrap().value()),
        ["jdoe@example.com", "jqpublic@example.com"]
    );
    assert_eq!(
        paths(attendee.delegators().unwrap().value()),
        ["boss@example.com"]
    );
    assert_eq!(
        attendee.sent_by().unwrap().value().uri().path(),
        "sray@example.com"
    );
    assert_eq!(
        attendee.directory().unwrap().value().host_str(),
        Some("example.com")
    );
}

/// RFC 5545 §3.8.4.3: `ORGANIZER` with `CN` and `SENT-BY`.
#[test]
fn organizer_exposes_address_cn_and_sent_by() {
    let c = calendar(&[
        "ORGANIZER;CN=John Smith;SENT-BY=\"mailto:jane_doe@example.com\":\
         mailto:jsmith@example.com",
    ]);
    let organizer = event(&c).organizer().unwrap();
    assert_eq!(organizer.value().uri().path(), "jsmith@example.com");
    assert_eq!(organizer.common_name().unwrap().as_str(), "John Smith");
    assert_eq!(
        organizer.sent_by().unwrap().value().uri().path(),
        "jane_doe@example.com"
    );
}

/// RFC 5545 §3.8.6.3: `TRIGGER;RELATED=END:PT5M` is a 5-minute offset from
/// the end.
#[test]
fn trigger_exposes_duration_and_related() {
    let c = calendar(&[
        "BEGIN:VALARM",
        "ACTION:DISPLAY",
        "DESCRIPTION:Reminder",
        "TRIGGER;RELATED=END:PT5M",
        "END:VALARM",
    ]);
    let trigger = event(&c).alarms()[0].trigger();
    let DateTimeDuration::Duration(d) = trigger.value() else {
        panic!("expected a DURATION trigger");
    };
    assert_eq!(d.num_minutes(), 5);
    assert!(matches!(
        trigger.related(),
        Some(AlarmTriggerRelationship::End)
    ));
}

/// RFC 5545 §3.8.6.3: a negative duration fires *before* its anchor, and an
/// unspecified `RELATED` is `None` (the reader applies the START default).
#[test]
fn trigger_negative_duration_and_absent_related() {
    let c = calendar(&[
        "BEGIN:VALARM",
        "ACTION:DISPLAY",
        "DESCRIPTION:Reminder",
        "TRIGGER:-PT15M",
        "END:VALARM",
    ]);
    let trigger = event(&c).alarms()[0].trigger();
    let DateTimeDuration::Duration(d) = trigger.value() else {
        panic!("expected a DURATION trigger");
    };
    assert_eq!(d.num_minutes(), -15);
    assert!(trigger.related().is_none());
}

/// RFC 5545 §3.8.6.3: `TRIGGER;VALUE=DATE-TIME:19980101T050000Z` is an
/// absolute UTC instant.
#[test]
fn trigger_exposes_an_absolute_datetime() {
    let c = calendar(&[
        "BEGIN:VALARM",
        "ACTION:DISPLAY",
        "DESCRIPTION:Reminder",
        "TRIGGER;VALUE=DATE-TIME:19980101T050000Z",
        "END:VALARM",
    ]);
    let trigger = event(&c).alarms()[0].trigger();
    let DateTimeDuration::DateTime(dt) = trigger.value() else {
        panic!("expected a DATE-TIME trigger");
    };
    assert_eq!(
        dt,
        &cratical::values::DateTime::Utc(
            Utc.with_ymd_and_hms(1998, 1, 1, 5, 0, 0).unwrap()
        )
    );
}

/// RFC 5545 §3.8.5.1: `EXDATE` is a comma-separated list of `DATE-TIME`s.
#[test]
fn exdate_exposes_its_values() {
    let c = calendar(&[
        "EXDATE:19960402T010000Z,19960403T010000Z,19960404T010000Z",
    ]);
    let exdate = &event(&c).exdate()[0];
    assert_eq!(exdate.value().len(), 3);
    assert!(
        exdate
            .value()
            .iter()
            .all(|v| matches!(v, DateOrDatetime::DateTime(_)))
    );
}

/// RFC 5545 §3.8.5.2: `RDATE` may mix `PERIOD` shapes; a client has to be
/// able to see a `PERIOD` in order to reject or model it.
#[test]
fn rdate_exposes_periods_and_plain_datetimes() {
    let c = calendar(&[
        "RDATE;VALUE=PERIOD:19960403T020000Z/19960403T040000Z,\
         19960404T010000Z/PT3H",
        "RDATE:19970714T123000Z",
    ]);
    let rdates = event(&c).rdate();
    assert_eq!(rdates.len(), 2);
    assert_eq!(rdates[0].value().len(), 2);
    assert!(
        rdates[0]
            .value()
            .iter()
            .all(|v| matches!(v, DateTimePeriod::Period(_)))
    );
    assert!(matches!(rdates[1].value(), [DateTimePeriod::DateTime(_)]));
}

/// RFC 5545 §3.8.1.2: `CATEGORIES:APPOINTMENT,EDUCATION`.
#[test]
fn categories_exposes_its_list() {
    let c = calendar(&["CATEGORIES:APPOINTMENT,EDUCATION"]);
    let categories = event(&c).categories()[0].value();
    let texts: Vec<&str> = categories.iter().map(|t| t.as_str()).collect();
    assert_eq!(texts, ["APPOINTMENT", "EDUCATION"]);
}

/// Parameter values built through their public constructors read back
/// unchanged.
#[test]
fn constructed_parameters_read_back_their_value() {
    assert_eq!(
        CommonName::new(Text::from("John Smith")).value().as_str(),
        "John Smith"
    );
    assert!(Rsvp::new(true).value());
    assert!(!Rsvp::new(false).value());
    assert_eq!(Label::new(Text::from("Video")).as_str(), "Video");
    assert_eq!(Language::new("en-US").unwrap().as_str(), "en-US");
    assert_eq!(
        Altrep::new(
            Uri::parse("CID:part3.msg.970415T083000@example.com").unwrap()
        )
        .value()
        .scheme(),
        "cid"
    );
    assert_eq!(
        DirectoryEntryReference::new(
            Uri::parse("ldap://example.com/o=ABC").unwrap()
        )
        .host_str(),
        Some("example.com")
    );

    let fmt = Fmttype::new(MediaType::new("application", "msword"));
    assert_eq!(fmt.value().media_type(), "application");
    assert_eq!(fmt.value().subtype(), "msword");

    assert!(matches!(
        Feature::new(vec![FeatureValue::Audio, FeatureValue::Chat]).value(),
        [FeatureValue::Audio, FeatureValue::Chat]
    ));
    assert!(matches!(
        ImageDisplay::new(vec![ImageDisplayValue::Badge]).value(),
        [ImageDisplayValue::Badge]
    ));
    assert_eq!(
        TimeZoneIdentifier::new(chrono_tz::America::New_York).as_str(),
        "America/New_York"
    );
}
