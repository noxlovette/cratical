//! Coverage for RFC 7953 (`VAVAILABILITY`), implemented under the
//! `rfc_7953` feature (default-enabled — see issue #16). These tests are
//! only compiled when that feature is active.
//!
//! The RFC-derived fixtures under
//! `tests/fixtures/collective-icalendar/availabilities/` and
//! `tests/fixtures/collective-icalendar/calendars/rfc_7953_3.ics` (see
//! `tests/out_of_scope.rs`) are bare excerpts — no `VCALENDAR` wrapper, or
//! missing `PRODID`/`VERSION` — so they can't be fed to `Calendar::parse`
//! as-is. This file reads those same untouched fixture files and wraps (or,
//! for the third, patches in the missing `PRODID`/`VERSION` of) their exact
//! bytes, rather than retyping the RFC's example text by hand.
//!
//! Hand-built bodies below are assembled with [`lines`] (joining with
//! CRLF) rather than a manually `\`-continued multi-line string literal —
//! `rustfmt` reflowing such a literal can shift a line break mid-escape
//! and corrupt its content (see `calendar::tests::
//! calendar_display_round_trips_a_minimal_calendar`'s pre-existing,
//! unrelated breakage for a real example of exactly that happening).

#![cfg(feature = "rfc_7953")]

use icalendar::{Calendar, Component};

fn fixture(rel: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/collective-icalendar/{rel}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {rel} should be readable: {e}"))
}

fn wrap(body: &str) -> Vec<u8> {
    format!(
        "BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\n{body}END:\
         VCALENDAR\r\n"
    )
    .into_bytes()
}

/// Joins `lines` with CRLF, plus a trailing CRLF — see this file's own docs
/// for why this is preferred over a `\`-continued string literal.
fn lines(lines: &[&str]) -> String {
    let mut s = lines.join("\r\n");
    s.push_str("\r\n");
    s
}

/// RFC 7953 §3.1's first example (`availabilities/rfc_7953_1.ics`): a
/// single `AVAILABLE` recurring weekly, wrapped in a minimal `VCALENDAR`.
#[test]
fn parses_the_rfc_example_1_fixture_and_reparses_after_display() {
    let body = fixture("availabilities/rfc_7953_1.ics");
    let calendar = Calendar::parse(&wrap(&body)).unwrap();
    assert_eq!(calendar.components().len(), 1);

    let Component::Availability(availability) = &calendar.components()[0]
    else {
        panic!("expected a VAVAILABILITY component");
    };
    assert_eq!(
        availability.uid().to_string(),
        "UID:0428C7D2-688E-4D2E-AC52-CD112E2469DF"
    );
    assert!(availability.dtstart().is_none());
    assert!(availability.dtend().is_none());
    // BUSYTYPE wasn't specified — RFC 7953 §3.2 says it then defaults to
    // BUSY-UNAVAILABLE, but that's a semantic default this crate leaves to
    // the caller to apply, not something `busytype()` fabricates.
    assert!(availability.busytype().is_none());

    assert_eq!(availability.available().len(), 1);
    let available = &availability.available()[0];
    assert_eq!(
        available.uid().to_string(),
        "UID:34EDA59B-6BB1-4E94-A66C-64999089C0AF"
    );
    // This fixture's AVAILABLE has no DTSTAMP, per RFC 7953's own example —
    // see Available's doc comment for why that's modeled as optional here.
    assert!(available.dtstamp().is_none());
    assert!(available.rrule().is_some());

    let rendered = calendar.to_string();
    let reparsed = Calendar::parse(rendered.as_bytes()).unwrap_or_else(|e| {
        panic!("rendered output failed to reparse: {e}\n---\n{rendered}")
    });
    assert_eq!(reparsed.components().len(), 1);
}

/// RFC 7953 §3.1's second example (`availabilities/rfc_7953_2.ics`): two
/// `AVAILABLE` subcomponents under one `VAVAILABILITY`, the enclosing
/// component itself carrying `DTSTART`/`DTEND`.
#[test]
fn parses_the_rfc_example_2_fixture() {
    let body = fixture("availabilities/rfc_7953_2.ics");
    let calendar = Calendar::parse(&wrap(&body)).unwrap();
    let Component::Availability(availability) = &calendar.components()[0]
    else {
        panic!("expected a VAVAILABILITY component");
    };
    assert!(availability.dtstart().is_some());
    assert!(availability.dtend().is_some());
    assert_eq!(availability.available().len(), 2);
    assert_eq!(
        availability.available()[0].location().unwrap().to_string(),
        "LOCATION:Main Office"
    );
    assert_eq!(
        availability.available()[1].location().unwrap().to_string(),
        "LOCATION:Branch Office"
    );
}

/// RFC 7953 §3.1's third example (`calendars/rfc_7953_3.ics`): three
/// top-level `VAVAILABILITY` components. Unlike the other two fixtures,
/// this one already has its own `BEGIN:VCALENDAR`/`END:VCALENDAR` wrapper
/// — it's only missing `PRODID`/`VERSION` (see `tests/out_of_scope.rs`) —
/// so this patches those in right after `BEGIN:VCALENDAR` instead of
/// wrapping.
#[test]
fn parses_the_rfc_example_3_fixture_once_prodid_and_version_are_present() {
    let original = fixture("calendars/rfc_7953_3.ics");
    let patched = original.replacen(
        "BEGIN:VCALENDAR\n",
        "BEGIN:VCALENDAR\nPRODID:-//example//EN\nVERSION:2.0\n",
        1,
    );
    let calendar = Calendar::parse(patched.as_bytes()).unwrap();
    assert_eq!(calendar.components().len(), 3);
    for component in calendar.components() {
        assert!(matches!(component, Component::Availability(_)));
    }
}

#[test]
fn busytype_parses_and_displays() {
    let body = lines(&[
        "BEGIN:VAVAILABILITY",
        "UID:busytype-test@example.com",
        "DTSTAMP:20111005T133225Z",
        "BUSYTYPE:BUSY",
        "BEGIN:AVAILABLE",
        "UID:avail-busytype-test@example.com",
        "DTSTART:20111002T090000Z",
        "END:AVAILABLE",
        "END:VAVAILABILITY",
    ]);

    let calendar = Calendar::parse(&wrap(&body)).unwrap();
    let Component::Availability(availability) = &calendar.components()[0]
    else {
        panic!("expected a VAVAILABILITY component");
    };
    assert_eq!(
        availability.busytype().unwrap().to_string(),
        "BUSYTYPE:BUSY"
    );
}

/// RFC 7953 §3.1: "Either 'dtend' or 'duration' MAY appear in an
/// 'availabilityprop', but 'dtend' and 'duration' MUST NOT occur in the
/// same 'availabilityprop'."
#[test]
fn rejects_dtend_and_duration_together_on_vavailability() {
    let body = lines(&[
        "BEGIN:VAVAILABILITY",
        "UID:mutex-test@example.com",
        "DTSTAMP:20111005T133225Z",
        "DTSTART:20111002T000000Z",
        "DTEND:20111003T000000Z",
        "DURATION:PT1H",
        "END:VAVAILABILITY",
    ]);

    assert!(Calendar::parse(&wrap(&body)).is_err());
}

/// RFC 7953 §3.1: the same mutual-exclusion rule applies to `AVAILABLE`'s
/// own `availableprop`.
#[test]
fn rejects_dtend_and_duration_together_on_available() {
    let body = lines(&[
        "BEGIN:VAVAILABILITY",
        "UID:mutex-test-2@example.com",
        "DTSTAMP:20111005T133225Z",
        "BEGIN:AVAILABLE",
        "UID:avail-mutex-test@example.com",
        "DTSTART:20111002T090000Z",
        "DTEND:20111002T170000Z",
        "DURATION:PT1H",
        "END:AVAILABLE",
        "END:VAVAILABILITY",
    ]);

    assert!(Calendar::parse(&wrap(&body)).is_err());
}

/// `AVAILABLE` requires `DTSTART` and `UID` (RFC 7953 §3.1's
/// `availableprop` — `DTSTAMP` is REQUIRED by the formal grammar too, but
/// modeled as optional here, see `Available`'s own docs) — missing
/// `DTSTART` must fail at `build()`.
#[test]
fn rejects_available_missing_dtstart() {
    let body = lines(&[
        "BEGIN:VAVAILABILITY",
        "UID:missing-dtstart@example.com",
        "DTSTAMP:20111005T133225Z",
        "BEGIN:AVAILABLE",
        "UID:avail-missing-dtstart@example.com",
        "END:AVAILABLE",
        "END:VAVAILABILITY",
    ]);

    assert!(Calendar::parse(&wrap(&body)).is_err());
}

/// `AVAILABLE` is only legal nested inside `VAVAILABILITY` (RFC 7953
/// §3.1) — under a `VEVENT`, it must be rejected the same way a `VALARM`
/// is rejected under a `VJOURNAL`.
#[test]
fn rejects_available_nested_under_vevent() {
    let body = lines(&[
        "BEGIN:VEVENT",
        "UID:not-a-vavailability@example.com",
        "DTSTAMP:20111005T133225Z",
        "DTSTART:20111002T090000Z",
        "BEGIN:AVAILABLE",
        "UID:avail-under-vevent@example.com",
        "DTSTART:20111002T090000Z",
        "END:AVAILABLE",
        "END:VEVENT",
    ]);

    assert!(Calendar::parse(&wrap(&body)).is_err());
}
