//! Coverage for RFC 5546 (iTIP), implemented under the `rfc_5546` feature
//! (not default-enabled — see `Cargo.toml`). These tests are only compiled
//! when that feature is active.
//!
//! Two kinds of coverage:
//!
//! - Hand-crafted minimal `VCALENDAR`s exercising [`icalendar::itip`]'s
//!   validation directly: one conformant and one violating example per
//!   method/component-type combination that's easy to get wrong.
//! - The real-world fixtures in `ITIP_NONCONFORMANT_FIXTURES` (see `build.rs`)
//!   — valid RFC 5545 objects that stamp a `METHOD` without satisfying that
//!   method's RFC 5546 restrictions. `build.rs` excludes them from the blanket
//!   "must parse successfully" fixture tests only when `rfc_5546` is enabled;
//!   this file pins down the specific reason each one is now rejected.

#![cfg(feature = "rfc_5546")]

use icalendar::{Calendar, CalendarParseError};

fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/tests/fixtures/collective-icalendar/calendars/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(&path)
        .unwrap_or_else(|e| panic!("fixture {name} should be readable: {e}"))
}

/// Asserts that parsing `bytes` fails with a message containing `needle` —
/// `CalendarParseError::Parse` wraps `ast::parser::ParseError`, which isn't
/// publicly nameable (see `tests/malformed_input.rs`'s own note on this),
/// so content is checked through `Display` rather than by matching the
/// concrete variant.
fn assert_rejected(bytes: &[u8], needle: &str) {
    let err = Calendar::parse(bytes).unwrap_err();
    assert!(
        matches!(err, CalendarParseError::Parse(_)),
        "expected a Parse error, got {err:?}"
    );
    let msg = err.to_string();
    assert!(
        msg.contains(needle),
        "expected error message to contain {needle:?}, got {msg:?}"
    );
}

fn wrap(method: &str, body: &str) -> Vec<u8> {
    format!(
        "BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\nMETHOD:\
         {method}\r\n{body}END:VCALENDAR\r\n"
    )
    .into_bytes()
}

// --- Method::recognized / StatusCode -------------------------------------

#[test]
fn method_recognized_matches_all_8_itip_tokens() {
    use icalendar::itip::Method;

    for (token, expected) in [
        ("PUBLISH", Method::Publish),
        ("REQUEST", Method::Request),
        ("REPLY", Method::Reply),
        ("ADD", Method::Add),
        ("CANCEL", Method::Cancel),
        ("REFRESH", Method::Refresh),
        ("COUNTER", Method::Counter),
        ("DECLINECOUNTER", Method::DeclineCounter),
    ] {
        let prop = icalendar::properties::Method::new(token.into());
        assert_eq!(Method::recognized(&prop), Some(expected), "{token}");
        assert_eq!(expected.to_string(), token);
    }
}

#[test]
fn method_recognized_rejects_a_non_itip_token() {
    use icalendar::itip::Method;

    let prop = icalendar::properties::Method::new("X-CUSTOM-METHOD".into());
    assert_eq!(Method::recognized(&prop), None);
}

#[test]
fn status_code_2_0_is_success() {
    use icalendar::itip::StatusCode;

    assert_eq!(StatusCode::Success.code(), (2, 0));
    assert_eq!(StatusCode::Success.description(), "Success");
    assert_eq!(StatusCode::Success.to_string(), "2.0");
}

#[test]
fn status_code_5_3_is_no_scheduling_support() {
    use icalendar::itip::StatusCode;

    assert_eq!(StatusCode::NoSchedulingSupport.code(), (5, 3));
    assert_eq!(StatusCode::NoSchedulingSupport.to_string(), "5.3");
}

// --- A METHOD naming something other than the 8 iTIP tokens is a no-op ---

#[test]
fn unrecognized_method_skips_itip_validation_entirely() {
    // No ORGANIZER, no ATTENDEE — would fail every VEVENT PUBLISH/REQUEST
    // table, but METHOD:X-CUSTOM isn't one of the 8 recognized tokens, so
    // RFC 5546 doesn't apply and this parses fine.
    let bytes = wrap(
        "X-CUSTOM",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nEND:VEVENT\r\n",
    );
    assert!(Calendar::parse(&bytes).is_ok());
}

// --- VEVENT ---------------------------------------------------------------

#[test]
fn vevent_publish_requires_organizer() {
    let bytes = wrap(
        "PUBLISH",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nSUMMARY:Team meeting\r\nEND:VEVENT\r\n",
    );
    assert_rejected(&bytes, "MUST include ORGANIZER for METHOD:PUBLISH");
}

#[test]
fn vevent_publish_rejects_an_attendee() {
    let bytes = wrap(
        "PUBLISH",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nSUMMARY:Team \
         meeting\r\nATTENDEE:mailto:b@example.com\r\nEND:VEVENT\r\n",
    );
    assert_rejected(&bytes, "MUST NOT include ATTENDEE for METHOD:PUBLISH");
}

#[test]
fn vevent_publish_valid_minimal_example_parses() {
    let bytes = wrap(
        "PUBLISH",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nSUMMARY:Team meeting\r\nEND:VEVENT\r\n",
    );
    assert!(Calendar::parse(&bytes).is_ok());
}

#[test]
fn vevent_request_requires_at_least_one_attendee() {
    let bytes = wrap(
        "REQUEST",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nSUMMARY:Team meeting\r\nEND:VEVENT\r\n",
    );
    assert_rejected(&bytes, "MUST include ATTENDEE for METHOD:REQUEST");
}

#[test]
fn vevent_request_valid_minimal_example_parses() {
    let bytes = wrap(
        "REQUEST",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nATTENDEE:mailto:b@example.com\r\nSUMMARY:Team \
         meeting\r\nEND:VEVENT\r\n",
    );
    assert!(Calendar::parse(&bytes).is_ok());
}

#[test]
fn vevent_reply_requires_exactly_one_attendee() {
    let bytes = wrap(
        "REPLY",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nORGANIZER:mailto:a@example.com\r\nATTENDEE:mailto:b@example.com\r\\
         nATTENDEE:mailto:c@example.com\r\nEND:VEVENT\r\n",
    );
    assert_rejected(
        &bytes,
        "ATTENDEE MUST occur exactly once for METHOD:REPLY",
    );
}

#[test]
fn vevent_reply_rejects_valarm() {
    let bytes = wrap(
        "REPLY",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nORGANIZER:mailto:a@example.com\r\nATTENDEE:mailto:b@example.com\r\\
         nBEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT15M\r\nDESCRIPTION:\
         Reminder\r\nEND:VALARM\r\nEND:VEVENT\r\n",
    );
    assert_rejected(&bytes, "MUST NOT include VALARM for METHOD:REPLY");
}

#[test]
fn vevent_add_requires_a_positive_sequence() {
    let bytes = wrap(
        "ADD",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nSUMMARY:Extra instance\r\nSEQUENCE:0\r\nEND:VEVENT\r\n",
    );
    assert_rejected(&bytes, "SEQUENCE MUST be greater than 0 for METHOD:ADD");
}

#[test]
fn vevent_add_rejects_more_than_one_vevent() {
    // Two distinct UIDs, so this reaches iTIP's own "ADD allows exactly one
    // VEVENT" check rather than tripping RFC 5545's unrelated
    // same-UID-same-instance rule first.
    let first = "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:\
                 20250101T000000Z\r\nDTSTART:20250101T100000Z\r\nORGANIZER:\
                 mailto:a@example.com\r\nSUMMARY:Extra \
                 instance\r\nSEQUENCE:1\r\nEND:VEVENT\r\n";
    let second = first.replace("UID:1@example.com", "UID:2@example.com");
    let bytes = wrap("ADD", &format!("{first}{second}"));
    assert_rejected(&bytes, "requires exactly one VEVENT component(s)");
}

#[test]
fn vevent_cancel_valid_minimal_example_parses() {
    let bytes = wrap(
        "CANCEL",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nORGANIZER:mailto:a@example.com\r\nSEQUENCE:1\r\nSTATUS:CANCELLED\r\\
         nEND:VEVENT\r\n",
    );
    assert!(Calendar::parse(&bytes).is_ok());
}

#[test]
fn vevent_refresh_rejects_a_description() {
    let bytes = wrap(
        "REFRESH",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nORGANIZER:mailto:a@example.com\r\nATTENDEE:mailto:b@example.com\r\\
         nDESCRIPTION:please resend\r\nEND:VEVENT\r\n",
    );
    assert_rejected(&bytes, "MUST NOT include DESCRIPTION for METHOD:REFRESH");
}

#[test]
fn vevent_counter_requires_a_summary() {
    let bytes = wrap(
        "COUNTER",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nSEQUENCE:0\r\nEND:VEVENT\r\n",
    );
    assert_rejected(&bytes, "MUST include SUMMARY for METHOD:COUNTER");
}

#[test]
fn vevent_declinecounter_requires_at_least_one_attendee() {
    let bytes = wrap(
        "DECLINECOUNTER",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nORGANIZER:mailto:a@example.com\r\nSEQUENCE:0\r\nEND:VEVENT\r\n",
    );
    assert_rejected(&bytes, "MUST include ATTENDEE for METHOD:DECLINECOUNTER");
}

#[test]
fn vevent_request_requires_matching_uid_across_components() {
    let bytes = wrap(
        "REQUEST",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nATTENDEE:mailto:b@example.com\r\nSUMMARY:Team \
         meeting\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:2@example.com\r\\
         nDTSTAMP:20250101T000000Z\r\nDTSTART:20250102T100000Z\r\nORGANIZER:\
         mailto:a@example.com\r\nATTENDEE:mailto:b@example.com\r\nSUMMARY:\
         Team meeting (recurrence)\r\nEND:VEVENT\r\n",
    );
    assert_rejected(
        &bytes,
        "all VEVENT components MUST share the same UID for METHOD:REQUEST",
    );
}

// --- VTODO ------------------------------------------------------------

#[test]
fn vtodo_publish_requires_priority() {
    let bytes = wrap(
        "PUBLISH",
        "BEGIN:VTODO\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nSUMMARY:Ship the report\r\nEND:VTODO\r\n",
    );
    assert_rejected(&bytes, "MUST include PRIORITY for METHOD:PUBLISH");
}

#[test]
fn vtodo_publish_valid_minimal_example_parses() {
    let bytes = wrap(
        "PUBLISH",
        "BEGIN:VTODO\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nPRIORITY:1\r\nSUMMARY:Ship the report\r\nEND:VTODO\r\n",
    );
    assert!(Calendar::parse(&bytes).is_ok());
}

#[test]
fn vtodo_refresh_rejects_organizer() {
    // Unlike VEVENT's REFRESH, RFC 5546's VTODO REFRESH table lists
    // ORGANIZER as `0` (MUST NOT be present) — an asymmetry worth its own
    // regression test.
    let bytes = wrap(
        "REFRESH",
        "BEGIN:VTODO\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nATTENDEE:mailto:b@example.com\r\nORGANIZER:mailto:a@example.com\r\\
         nEND:VTODO\r\n",
    );
    assert_rejected(&bytes, "MUST NOT include ORGANIZER for METHOD:REFRESH");
}

#[test]
fn vtodo_refresh_valid_minimal_example_parses() {
    let bytes = wrap(
        "REFRESH",
        "BEGIN:VTODO\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nATTENDEE:mailto:b@example.com\r\nEND:VTODO\r\n",
    );
    assert!(Calendar::parse(&bytes).is_ok());
}

// --- VJOURNAL -----------------------------------------------------------

#[test]
fn vjournal_only_supports_publish_add_cancel() {
    let bytes = wrap(
        "REQUEST",
        "BEGIN:VJOURNAL\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nDESCRIPTION:Minutes\r\nEND:VJOURNAL\r\n",
    );
    assert_rejected(&bytes, "METHOD:REQUEST is not defined for VJOURNAL");
}

#[test]
fn vjournal_publish_valid_minimal_example_parses() {
    let bytes = wrap(
        "PUBLISH",
        "BEGIN:VJOURNAL\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nDESCRIPTION:Minutes\r\nEND:VJOURNAL\r\n",
    );
    assert!(Calendar::parse(&bytes).is_ok());
}

#[test]
fn vjournal_cancel_status_must_be_cancelled_if_present() {
    let bytes = wrap(
        "CANCEL",
        "BEGIN:VJOURNAL\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nORGANIZER:mailto:a@example.com\r\nSEQUENCE:1\r\nSTATUS:DRAFT\r\nEND:\
         VJOURNAL\r\n",
    );
    assert_rejected(
        &bytes,
        "STATUS MUST be one of CANCELLED for METHOD:CANCEL",
    );
}

// --- VFREEBUSY ----------------------------------------------------------

#[test]
fn vfreebusy_publish_requires_organizer() {
    let bytes = wrap(
        "PUBLISH",
        "BEGIN:VFREEBUSY\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T000000Z\r\nDTEND:20250102T000000Z\r\nEND:VFREEBUSY\\
         r\n",
    );
    assert_rejected(&bytes, "MUST include ORGANIZER for METHOD:PUBLISH");
}

#[test]
fn vfreebusy_publish_valid_minimal_example_parses() {
    let bytes = wrap(
        "PUBLISH",
        "BEGIN:VFREEBUSY\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T000000Z\r\nDTEND:20250102T000000Z\r\nORGANIZER:\
         mailto:a@example.com\r\nEND:VFREEBUSY\r\n",
    );
    assert!(Calendar::parse(&bytes).is_ok());
}

#[test]
fn vfreebusy_request_rejects_a_freebusy_property() {
    let bytes = wrap(
        "REQUEST",
        "BEGIN:VFREEBUSY\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T000000Z\r\nDTEND:20250102T000000Z\r\nORGANIZER:\
         mailto:a@example.com\r\nATTENDEE:mailto:b@example.com\r\nFREEBUSY:\
         20250101T090000Z/20250101T100000Z\r\nEND:VFREEBUSY\r\n",
    );
    assert_rejected(&bytes, "MUST NOT include FREEBUSY for METHOD:REQUEST");
}

#[test]
fn vfreebusy_does_not_support_cancel() {
    let bytes = wrap(
        "CANCEL",
        "BEGIN:VFREEBUSY\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T000000Z\r\nDTEND:20250102T000000Z\r\nORGANIZER:\
         mailto:a@example.com\r\nEND:VFREEBUSY\r\n",
    );
    assert_rejected(&bytes, "METHOD:CANCEL is not defined for VFREEBUSY");
}

// --- Mixed component types ------------------------------------------------

#[test]
fn mixing_component_types_under_one_method_is_rejected() {
    let bytes = wrap(
        "PUBLISH",
        "BEGIN:VEVENT\r\nUID:1@example.com\r\nDTSTAMP:20250101T000000Z\r\\
         nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@example.com\r\\
         nSUMMARY:Team \
         meeting\r\nEND:VEVENT\r\nBEGIN:VTODO\r\nUID:2@example.com\r\nDTSTAMP:\
         20250101T000000Z\r\nDTSTART:20250101T100000Z\r\nORGANIZER:mailto:a@\
         example.com\r\nPRIORITY:1\r\nSUMMARY:Ship the report\r\nEND:VTODO\r\n",
    );
    assert_rejected(
        &bytes,
        "METHOD:PUBLISH MUST NOT mix component types in one VCALENDAR",
    );
}

// --- Real-world fixtures that stamp METHOD without iTIP conformance ------

#[test]
fn alarm_etar_future_rejected_for_missing_organizer() {
    let bytes = fixture("alarm_etar_future.ics");
    assert_rejected(&bytes, "MUST include ORGANIZER for METHOD:PUBLISH");
}

#[test]
fn alarm_etar_notification_rejected_for_missing_organizer() {
    let bytes = fixture("alarm_etar_notification.ics");
    assert_rejected(&bytes, "MUST include ORGANIZER for METHOD:PUBLISH");
}

#[test]
fn alarm_etar_notification_clicked_rejected_for_missing_organizer() {
    let bytes = fixture("alarm_etar_notification_clicked.ics");
    assert_rejected(&bytes, "MUST include ORGANIZER for METHOD:PUBLISH");
}

#[test]
fn alarm_google_acknowledged_rejected_for_missing_organizer() {
    let bytes = fixture("alarm_google_acknowledged.ics");
    assert_rejected(&bytes, "MUST include ORGANIZER for METHOD:PUBLISH");
}

#[test]
fn alarm_google_future_rejected_for_missing_organizer() {
    let bytes = fixture("alarm_google_future.ics");
    assert_rejected(&bytes, "MUST include ORGANIZER for METHOD:PUBLISH");
}

#[test]
fn issue_350_rejected_for_missing_attendee() {
    let bytes = fixture("issue_350.ics");
    assert_rejected(&bytes, "MUST include ATTENDEE for METHOD:REQUEST");
}

#[test]
fn issue_836_do_not_quote_tzid_rejected_for_missing_organizer() {
    let bytes = fixture("issue_836_do_not_quote_tzid.ics");
    assert_rejected(&bytes, "MUST include ORGANIZER for METHOD:PUBLISH");
}

#[test]
fn timezone_same_start_rejected_for_missing_attendee() {
    let bytes = fixture("timezone_same_start.ics");
    assert_rejected(&bytes, "MUST include ATTENDEE for METHOD:REQUEST");
}

#[test]
fn x_location_rejected_for_missing_organizer() {
    let bytes = fixture("x_location.ics");
    assert_rejected(&bytes, "MUST include ORGANIZER for METHOD:PUBLISH");
}
