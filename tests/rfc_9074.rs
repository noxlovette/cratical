//! Coverage for RFC 9073 (`VLOCATION`, nested inside `VALARM`) and the RFC
//! 9074 `VALARM` extensions (`UID`, `RELATED-TO`, `ACKNOWLEDGED`,
//! `PROXIMITY`), implemented under the `rfc-9074` feature (default-enabled
//! — see issue #17). These tests are only compiled when that feature is
//! active.
//!
//! The RFC-derived fixtures under
//! `tests/fixtures/collective-icalendar/events/rfc_9074_*.ics` (see
//! `tests/out_of_scope.rs`) are bare `VEVENT` excerpts — no `VCALENDAR`
//! wrapper — so they can't be fed to `Calendar::parse` as-is. This file
//! reads those same untouched fixture files and wraps their exact bytes
//! (minus the blank separator lines RFC 9074 §6's typeset examples use
//! between properties, which aren't valid content lines) in a minimal
//! `VCALENDAR`, rather than retyping the RFC's example text by hand.

#![cfg(feature = "rfc-9074")]

use cratical::Component;

fn fixture(rel: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/collective-icalendar/{rel}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {rel} should be readable: {e}"))
}

/// Reads `rel` and drops its blank separator lines, joining what's left
/// with CRLF (plus a trailing CRLF) — see this file's own docs for why.
fn body(rel: &str) -> String {
    let mut s: String = fixture(rel)
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\r\n");
    s.push_str("\r\n");
    s
}

fn wrap(body: &str) -> Vec<u8> {
    format!(
        "BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\n{body}END:\
         VCALENDAR\r\n"
    )
    .into_bytes()
}

/// Joins `lines` with CRLF, plus a trailing CRLF.
fn lines(lines: &[&str]) -> String {
    let mut s = lines.join("\r\n");
    s.push_str("\r\n");
    s
}

/// RFC 9074 §6's first example (`rfc_9074_example_1.ics`): a single
/// `VALARM` carrying the RFC 9074 `UID`.
#[test]
fn parses_the_rfc_example_1_fixture_and_reparses_after_display() {
    let calendar = cratical::Calendar::parse(&wrap(&body(
        "events/rfc_9074_example_1.ics",
    )))
    .unwrap();
    assert_eq!(calendar.components().len(), 1);

    let Component::Event(event) = &calendar.components()[0] else {
        panic!("expected a VEVENT component");
    };
    assert_eq!(event.alarms().len(), 1);
    let alarm = &event.alarms()[0];
    assert_eq!(
        alarm.uid().unwrap().to_string(),
        "UID:8297C37D-BA2D-4476-91AE-C1EAA364F8E1"
    );
    assert!(alarm.acknowledged().is_none());

    let rendered = calendar.to_string();
    let reparsed = cratical::Calendar::parse(rendered.as_bytes())
        .unwrap_or_else(|e| {
            panic!("rendered output failed to reparse: {e}\n---\n{rendered}")
        });
    assert_eq!(reparsed.components().len(), 1);
}

/// RFC 9074 §6's second example (`rfc_9074_example_2.ics`): the original
/// `VALARM` now carries `ACKNOWLEDGED`, and a sibling "snooze" `VALARM`
/// relates back to it via `RELATED-TO;RELTYPE=SNOOZE`.
#[test]
fn parses_the_rfc_example_2_fixture() {
    let calendar = cratical::Calendar::parse(&wrap(&body(
        "events/rfc_9074_example_2.ics",
    )))
    .unwrap();
    let Component::Event(event) = &calendar.components()[0] else {
        panic!("expected a VEVENT component");
    };
    assert_eq!(event.alarms().len(), 2);

    let original = &event.alarms()[0];
    assert!(original.acknowledged().is_some());

    let snooze = &event.alarms()[1];
    assert_eq!(snooze.related().len(), 1);
    assert_eq!(
        snooze.related()[0].to_string(),
        "RELATED-TO;RELTYPE=SNOOZE:8297C37D-BA2D-4476-91AE-C1EAA364F8E1"
    );
}

/// RFC 9074 §6's third example (`rfc_9074_example_3.ics`): same shape as
/// the second, with a later `ACKNOWLEDGED` timestamp and a new snooze
/// `VALARM`.
#[test]
fn parses_the_rfc_example_3_fixture() {
    let calendar = cratical::Calendar::parse(&wrap(&body(
        "events/rfc_9074_example_3.ics",
    )))
    .unwrap();
    let Component::Event(event) = &calendar.components()[0] else {
        panic!("expected a VEVENT component");
    };
    assert_eq!(event.alarms().len(), 2);
    assert!(event.alarms()[0].acknowledged().is_some());
    assert_eq!(event.alarms()[1].related().len(), 1);
}

/// RFC 9074 §6's fourth example (`rfc_9074_example_4.ics`): both the
/// original and the snooze `VALARM` now carry `ACKNOWLEDGED`.
#[test]
fn parses_the_rfc_example_4_fixture() {
    let calendar = cratical::Calendar::parse(&wrap(&body(
        "events/rfc_9074_example_4.ics",
    )))
    .unwrap();
    let Component::Event(event) = &calendar.components()[0] else {
        panic!("expected a VEVENT component");
    };
    assert_eq!(event.alarms().len(), 2);
    assert!(event.alarms()[0].acknowledged().is_some());
    assert!(event.alarms()[1].acknowledged().is_some());
}

/// RFC 9074 §8.2's proximity example (`rfc_9074_example_proximity.ics`): a
/// `VALARM` with `PROXIMITY:DEPART` and a nested `VLOCATION` specifying the
/// location via a `geo:` `URL`. Unlike the other four fixtures, this one's
/// `VEVENT` carries no `UID`/`DTSTAMP`/`DTSTART` at all — it's a minimal
/// excerpt demonstrating just the alarm — so those are patched in here the
/// same way `tests/rfc_7953.rs` patches its third fixture's missing
/// `PRODID`/`VERSION`.
#[test]
fn parses_the_proximity_example_fixture_with_nested_vlocation() {
    let patched = body("events/rfc_9074_example_proximity.ics").replacen(
        "BEGIN:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:proximity-test@example.com\r\nDTSTAMP:\
         19760401T005545Z\r\nDTSTART:19760401T005545Z\r\n",
        1,
    );
    let calendar = cratical::Calendar::parse(&wrap(&patched)).unwrap();
    let Component::Event(event) = &calendar.components()[0] else {
        panic!("expected a VEVENT component");
    };
    assert_eq!(event.alarms().len(), 1);
    let alarm = &event.alarms()[0];

    assert_eq!(alarm.proximity().unwrap().to_string(), "PROXIMITY:DEPART");

    assert_eq!(alarm.locations().len(), 1);
    let location = &alarm.locations()[0];
    assert_eq!(location.uid().to_string(), "UID:123456-abcdef-98765432");
    assert_eq!(location.name().unwrap().to_string(), "NAME:Office");
    assert_eq!(
        location.url().unwrap().to_string(),
        "URL:geo:40.443,-79.945;u=10"
    );

    let rendered = calendar.to_string();
    let reparsed = cratical::Calendar::parse(rendered.as_bytes())
        .unwrap_or_else(|e| {
            panic!("rendered output failed to reparse: {e}\n---\n{rendered}")
        });
    let Component::Event(event) = &reparsed.components()[0] else {
        panic!("expected a VEVENT component");
    };
    assert_eq!(event.alarms()[0].locations().len(), 1);
}

/// RFC 9074 §8: a `VLOCATION` is only legal inside a `VALARM` that also
/// carries `PROXIMITY` — `AlarmBuilder::build` must reject one attached
/// without it.
#[test]
fn rejects_vlocation_without_proximity() {
    let body = lines(&[
        "BEGIN:VEVENT",
        "UID:no-proximity@example.com",
        "DTSTAMP:20210302T151004Z",
        "DTSTART:20210302T103000Z",
        "BEGIN:VALARM",
        "ACTION:DISPLAY",
        "TRIGGER:-PT15M",
        "DESCRIPTION:Event reminder",
        "BEGIN:VLOCATION",
        "UID:loc@example.com",
        "END:VLOCATION",
        "END:VALARM",
        "END:VEVENT",
    ]);

    assert!(cratical::Calendar::parse(&wrap(&body)).is_err());
}

/// `VLOCATION` requires a `UID` (RFC 9073 §7.2's `locprop`).
#[test]
fn rejects_vlocation_missing_uid() {
    let body = lines(&[
        "BEGIN:VEVENT",
        "UID:missing-uid@example.com",
        "DTSTAMP:20210302T151004Z",
        "DTSTART:20210302T103000Z",
        "BEGIN:VALARM",
        "ACTION:DISPLAY",
        "TRIGGER:-PT15M",
        "DESCRIPTION:Event reminder",
        "PROXIMITY:ARRIVE",
        "BEGIN:VLOCATION",
        "NAME:Office",
        "END:VLOCATION",
        "END:VALARM",
        "END:VEVENT",
    ]);

    assert!(cratical::Calendar::parse(&wrap(&body)).is_err());
}

/// `VLOCATION` is only legal nested inside `VALARM` — under a `VEVENT`
/// directly, it must be rejected the same way any other unrecognized
/// nested `BEGIN` would be.
#[test]
fn rejects_vlocation_nested_directly_under_vevent() {
    let body = lines(&[
        "BEGIN:VEVENT",
        "UID:not-a-valarm@example.com",
        "DTSTAMP:20210302T151004Z",
        "DTSTART:20210302T103000Z",
        "BEGIN:VLOCATION",
        "UID:loc@example.com",
        "END:VLOCATION",
        "END:VEVENT",
    ]);

    assert!(cratical::Calendar::parse(&wrap(&body)).is_err());
}

/// `LOCATION-TYPE` accepts a COMMA-separated list of values (RFC 9073
/// §6.1).
#[test]
fn location_type_parses_and_displays_multiple_values() {
    let body = lines(&[
        "BEGIN:VEVENT",
        "UID:loctype-test@example.com",
        "DTSTAMP:20210302T151004Z",
        "DTSTART:20210302T103000Z",
        "BEGIN:VALARM",
        "ACTION:DISPLAY",
        "TRIGGER:-PT15M",
        "DESCRIPTION:Event reminder",
        "PROXIMITY:ARRIVE",
        "BEGIN:VLOCATION",
        "UID:loc@example.com",
        "LOCATION-TYPE:HOTEL,RESTAURANT",
        "END:VLOCATION",
        "END:VALARM",
        "END:VEVENT",
    ]);

    let calendar = cratical::Calendar::parse(&wrap(&body)).unwrap();
    let Component::Event(event) = &calendar.components()[0] else {
        panic!("expected a VEVENT component");
    };
    let location = &event.alarms()[0].locations()[0];
    assert_eq!(
        location.loctype().unwrap().to_string(),
        "LOCATION-TYPE:HOTEL,RESTAURANT"
    );
}
