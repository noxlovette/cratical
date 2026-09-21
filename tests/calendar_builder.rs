//! Black-box coverage that a `Calendar` can be *constructed* from outside the
//! crate — `CalendarBuilder` plus `Calendar::try_from` — and is held to the
//! same RFC 5545 rules a parsed one is. Everything here goes through the
//! public API only; if a test can't be written from outside the crate, that
//! is a gap in `src/`, not something to work around.

use cratical::{
    Calendar, CalendarBuilder, Component, ComponentError,
    components::{
        event::{Event, EventBuilder},
        timezone::{Timezone, TimezoneBuilder, TzPropBuilder},
    },
    properties::{
        DateTimeStamp, DateTimeStart, Method, ProductIdentifier,
        TimeZoneIdentifier, TimeZoneOffsetFrom, TimeZoneOffsetTo, Uid, Version,
    },
};

fn prodid() -> ProductIdentifier {
    ProductIdentifier::try_from(b":-//example//EN".as_slice()).unwrap()
}

fn version() -> Version {
    Version::try_from(b":2.0".as_slice()).unwrap()
}

fn base() -> CalendarBuilder {
    CalendarBuilder::new(prodid(), version())
}

/// A `VEVENT` with a `DTSTART`, so it's valid with or without a `METHOD`.
fn event(uid: &str) -> Event {
    EventBuilder::new(
        DateTimeStamp::try_from(b":19970901T130000Z".as_slice()).unwrap(),
        Uid::try_from(format!(":{uid}").as_bytes()).unwrap(),
    )
    .dtstart(DateTimeStart::try_from(b":19970903T163000Z".as_slice()).unwrap())
    .build(false)
    .unwrap()
}

fn event_without_dtstart(uid: &str) -> Event {
    EventBuilder::new(
        DateTimeStamp::try_from(b":19970901T130000Z".as_slice()).unwrap(),
        Uid::try_from(format!(":{uid}").as_bytes()).unwrap(),
    )
    .build(true)
    .unwrap()
}

fn timezone(tzid: &str) -> Timezone {
    let standard = TzPropBuilder::new(
        DateTimeStart::try_from(b":19671029T020000".as_slice()).unwrap(),
        TimeZoneOffsetTo::try_from(b":-0500".as_slice()).unwrap(),
        TimeZoneOffsetFrom::try_from(b":-0400".as_slice()).unwrap(),
    )
    .build()
    .unwrap();
    TimezoneBuilder::new(
        TimeZoneIdentifier::try_from(format!(":{tzid}").as_bytes()).unwrap(),
    )
    .standard(standard)
    .build()
    .unwrap()
}

#[test]
fn calendar_is_constructible_from_a_builder_via_try_from() {
    let calendar = Calendar::try_from(base().component(event("a@example.com")))
        .expect("PRODID, VERSION and one VEVENT is a valid VCALENDAR");
    assert_eq!(calendar.components().len(), 1);
    assert!(matches!(calendar.components()[0], Component::Event(_)));
}

#[test]
fn build_and_try_from_agree() {
    let via_build = base().component(event("a@example.com")).build().unwrap();
    let via_try_from =
        Calendar::try_from(base().component(event("a@example.com"))).unwrap();
    assert_eq!(via_build.to_string(), via_try_from.to_string());
}

#[test]
fn constructed_calendar_renders_as_a_reparseable_vcalendar() {
    let calendar = base()
        .component(timezone("America/New_York"))
        .component(event("a@example.com"))
        .component(event("b@example.com"))
        .build()
        .unwrap();
    let rendered = calendar.to_string();

    assert!(rendered.starts_with("BEGIN:VCALENDAR\r\n"));
    assert!(rendered.ends_with("END:VCALENDAR\r\n"));
    let reparsed = Calendar::parse(rendered.as_bytes())
        .unwrap_or_else(|e| panic!("should reparse: {e}\n---\n{rendered}"));
    assert_eq!(reparsed.components().len(), 3);
    assert_eq!(reparsed.to_string(), rendered);
}

#[test]
fn a_vcalendar_needs_at_least_one_component() {
    // RFC 5545 §3.6: "one or more calendar components".
    assert!(matches!(
        Calendar::try_from(base()),
        Err(ComponentError::RequiresAtLeastOne("VCALENDAR", _))
    ));
}

#[test]
fn vevent_needs_dtstart_unless_the_calendar_has_a_method() {
    // RFC 5545 §3.6.1: DTSTART is REQUIRED on VEVENT when METHOD is absent.
    assert!(matches!(
        Calendar::try_from(base().component(event_without_dtstart("a@x"))),
        Err(ComponentError::MissingField("DTSTART"))
    ));
    assert!(
        Calendar::try_from(
            base()
                .method(Method::try_from(b":X-CUSTOM".as_slice()).unwrap())
                .component(event_without_dtstart("a@x"))
        )
        .is_ok()
    );
}

#[test]
fn two_components_may_not_claim_the_same_uid_and_recurrence_id() {
    let result = Calendar::try_from(
        base()
            .component(event("same@example.com"))
            .component(event("same@example.com")),
    );
    assert!(matches!(result, Err(ComponentError::DuplicateUid(_))));
}

#[test]
fn vtimezones_must_be_unique() {
    // RFC 5545 §3.6.5: multiple VTIMEZONEs "MUST represent a unique time
    // zone definition".
    let result = Calendar::try_from(
        base()
            .component(timezone("America/New_York"))
            .component(timezone("America/New_York"))
            .component(event("a@example.com")),
    );
    assert!(matches!(result, Err(ComponentError::DuplicateTimeZone(_))));
}

#[cfg(feature = "rfc-5546")]
#[test]
fn itip_method_restrictions_apply_to_a_constructed_calendar() {
    // RFC 5546 §3.2.1: a PUBLISH VEVENT MUST carry ORGANIZER.
    let result = Calendar::try_from(
        base()
            .method(Method::try_from(b":PUBLISH".as_slice()).unwrap())
            .component(event("a@example.com")),
    );
    assert!(result.is_err());
}
