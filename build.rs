//! Generates one `#[test]` per file under `tests/fixtures/` so `cargo test`/
//! `cargo nextest run` reports each fixture individually (see issue #1:
//! wiring the fixture corpus up to the parser rather than leaving it inert).
//!
//! Two families are generated into `$OUT_DIR/fixture_tests.rs`, included by
//! `tests/fixtures.rs`:
//!
//! - `rfc5545/`, `rrule/`, `libical/`, `collective-icalendar/` (`.ics` files):
//!   each must parse successfully via `Calendar::parse` — these are real-world
//!   or spec-derived calendars, not deliberately-broken input.
//! - `libical-fuzz-corpus/` (non-`.txt` files, mostly `.bin`): fuzzer-found
//!   inputs. These are only asserted not to panic — a parse `Err` is a fine,
//!   expected outcome for adversarial/malformed bytes.
//!
//! A handful of files under `collective-icalendar/`/`libical/` are
//! themselves deliberately malformed (see issue #4) or otherwise genuinely
//! RFC 5545-invalid (missing a required property, an empty `VCALENDAR` with
//! zero components, malformed parameter syntax, ... — see issue #27) rather
//! than real-world-valid — they are excluded from the blanket "must parse
//! successfully" generation below (`MALFORMED_FIXTURES`) and instead get
//! individual, specific assertions in `tests/malformed_input.rs`.
//!
//! Another handful are excerpts too incomplete to form a valid
//! `icalobject` on their own: the three RFC 7953 `VAVAILABILITY` fixtures
//! and the five RFC 9074 `VALARM`-extension fixtures (bare excerpts with
//! no `VCALENDAR` wrapper, or missing `PRODID`/`VERSION`; `VAVAILABILITY`
//! is implemented under the `rfc_7953` feature (issue #16) and `VLOCATION`/
//! the RFC 9074 `VALARM` extensions are implemented under the `rfc_9074`
//! feature (issue #17)), plus a much larger set of bare component excerpts
//! (`VALARM`/`VEVENT`/`VTODO`/`VJOURNAL`/`VFREEBUSY`, and one bare `VCARD`
//! that isn't even an iCalendar object) with no `VCALENDAR` wrapper at all,
//! unrelated to any particular RFC feature's scope (issue #27) — excluded
//! the same way (`OUT_OF_SCOPE_FIXTURES`), with dedicated assertions in
//! `tests/out_of_scope.rs` and real coverage of the wrapped equivalents in
//! `tests/rfc_7953.rs`/`tests/rfc_9074.rs` (for the RFC 7953/9074 ones).

use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Files under `collective-icalendar/`/`libical/` that are deliberately
/// malformed (real-world-broken exports, RFC violations, fuzzer-found edge
/// cases — see issue #4) or otherwise genuinely RFC 5545-invalid content
/// (missing a required property, zero components under `VCALENDAR`,
/// malformed parameter syntax, duplicated singleton properties, ... — see
/// issue #27), not valid calendars that happen to fail. Excluded from the
/// blanket "must parse successfully" generation; each gets a dedicated,
/// specific test in `tests/malformed_input.rs` instead.
const MALFORMED_FIXTURES: &[&str] = &[
    "collective-icalendar/calendars/broken_ical.ics",
    "collective-icalendar/calendars/broken_dtstart.ics",
    "collective-icalendar/calendars/issue_1081_invalid_start_and_end.ics",
    "collective-icalendar/calendars/invalid_duration.ics",
    "collective-icalendar/calendars/issue_1081_invalid_rrule_freq.ics",
    "collective-icalendar/calendars/bom_calendar.ics",
    "collective-icalendar/calendars/big_bad_calendar.ics",
    "collective-icalendar/calendars/small_bad_calendar.ics",
    "collective-icalendar/calendars/parsing_error.ics",
    "collective-icalendar/calendars/parsing_error_in_UTC_offset.ics",
    "collective-icalendar/calendars/fuzz_testcase_0_char_in_component_name.ics",
    "collective-icalendar/calendars/fuzz_testcase_invalid_month.ics",
    "collective-icalendar/calendars/fuzz_testcase_vtimezone_lone_cr.ics",
    "collective-icalendar/calendars/\
     issue_351_whitespace_in_property_and_params.ics",
    // --- issue #27 bucket 2: genuinely RFC 5545-invalid content ---
    // Missing DTSTAMP (REQUIRED, e.g. §3.6.1).
    "collective-icalendar/calendars/created_calendar_with_unicode_fields.ics",
    "collective-icalendar/calendars/example.ics",
    "collective-icalendar/calendars/issue_1050_calendar_with_events_and_todos.\
     ics",
    "collective-icalendar/calendars/issue_1050_forward_timezone_reference.ics",
    "collective-icalendar/calendars/issue_1050_simple_calendar.ics",
    "collective-icalendar/calendars/issue_1081_event_with_rrule.ics",
    "collective-icalendar/calendars/issue_1081_list_of_properties.ics",
    "collective-icalendar/calendars/issue_1081_tzid_param.ics",
    "collective-icalendar/calendars/issue_1231_recurrence.ics",
    "collective-icalendar/calendars/issue_1426.ics",
    "collective-icalendar/calendars/issue_1426_value_parameters.ics",
    "libical/2445.ics",
    "libical/large.ics",
    // Missing PRODID (REQUIRED at VCALENDAR level, §3.4).
    "collective-icalendar/calendars/issue_168_expected_output.ics",
    "collective-icalendar/calendars/issue_178_custom_component_inside_other.\
     ics",
    "collective-icalendar/calendars/issue_322_expected_calendar.ics",
    "collective-icalendar/calendars/issue_722_missing_timezones.ics",
    "collective-icalendar/calendars/issue_798_freebusy.ics",
    "collective-icalendar/calendars/issue_798_related_to.ics",
    "collective-icalendar/calendars/period_with_timezone.ics",
    "collective-icalendar/calendars/rfc_5545_RDATE_example.ics",
    "collective-icalendar/calendars/rfc_6868.ics",
    "collective-icalendar/calendars/rfc_7256_multi_value_parameters.ics",
    "collective-icalendar/calendars/rfc_7986_conferences.ics",
    "collective-icalendar/calendars/rfc_7986_image.ics",
    "libical/smallcluster.ics",
    // Missing DTSTART (REQUIRED per §3.6.1's grammar when METHOD is absent
    // — confirmed none of these set METHOD at the calendar level).
    "collective-icalendar/calendars/issue_1050_multiple_calendars.ics",
    "collective-icalendar/calendars/issue_1050_uid_in_description.ics",
    "collective-icalendar/calendars/issue_1081_with_summary.ics",
    "collective-icalendar/calendars/issue_1549_binary_attachment.ics",
    "collective-icalendar/calendars/rfc_9253_gap.ics",
    "collective-icalendar/calendars/rfc_9253_related_to.ics",
    // Premature end of input: an empty RDATE value (grammar requires >=1
    // rdtval), a FREEBUSY/RDATE PERIOD using bare DATE instead of the
    // required DATE-TIME on both sides, a truncated DATE-TIME missing its
    // seconds component, or (restriction.ics) deliberately duplicated
    // singleton properties.
    "collective-icalendar/calendars/issue_1081_empty_rdate.ics",
    "collective-icalendar/calendars/issue_1633_freebusy_with_dates.ics",
    "collective-icalendar/calendars/issue_1633_rdate_with_dates.ics",
    "collective-icalendar/calendars/issue_1633_rdate_with_dates_and_tzid.ics",
    "libical/2.ics",
    "libical/process-calendar.ics",
    "libical/process-incoming.ics",
    "libical/restriction.ics",
    // Zero components under VCALENDAR (grammar requires >=1) or a
    // malformed VCALENDAR closing (trailing garbage line, missing END).
    "collective-icalendar/calendars/calendar_with_unicode.ics",
    "collective-icalendar/calendars/empty.ics",
    "collective-icalendar/calendars/issue_104_broken_calendar.ics",
    "collective-icalendar/calendars/issue_1050_empty_calendar.ics",
    "collective-icalendar/calendars/issue_1238.ics",
    "collective-icalendar/calendars/pr_480_summary_with_colon.ics",
    "collective-icalendar/calendars/rfc_7265_example_1.ics",
    "collective-icalendar/calendars/rfc_7986_properties.ics",
    "collective-icalendar/calendars/time.ics",
    // Malformed parameter syntax.
    "collective-icalendar/calendars/issue_168_input.ics",
    "collective-icalendar/calendars/issue_348_exception_parsing_value.ics",
    // Missing DESCRIPTION on a VALARM with ACTION:DISPLAY (required, §3.6.6).
    "collective-icalendar/calendars/issue_1050_all_components.ics",
    // Invalid UTF-8 / control characters.
    "collective-icalendar/calendars/issue_1081_invalid_start_valid_end.ics",
    // Malformed IMAGE value.
    "collective-icalendar/calendars/issue_1561_image_value.ics",
];

/// Files that are real-world-valid but too incomplete to form a valid
/// `icalobject` on their own — every one is either a bare component excerpt
/// with no `BEGIN:VCALENDAR` wrapper at all, or missing `PRODID`/`VERSION`.
/// Excluded from the blanket "must parse successfully" generation for that
/// reason; each gets a dedicated assertion in `tests/out_of_scope.rs`
/// instead.
///
/// Two sub-groups:
/// - RFC 7953 (`VAVAILABILITY`, `rfc_7953` feature, issue #16) and RFC
///   9073/9074 (`VLOCATION`/`VALARM` extensions, `rfc_9074` feature, issue #17)
///   excerpts — this crate *does* implement the component/properties they
///   exercise; real parsing coverage of the wrapped equivalent lives in
///   `tests/rfc_7953.rs`/`tests/rfc_9074.rs`.
/// - A much larger set of bare `VALARM`/`VEVENT`/`VTODO`/`VJOURNAL`/
///   `VFREEBUSY` excerpts (plus one bare `VCARD`, `libical/issue339.ics`, which
///   isn't even an iCalendar object) unrelated to any particular RFC feature's
///   scope — added per issue #27.
const OUT_OF_SCOPE_FIXTURES: &[&str] = &[
    "collective-icalendar/availabilities/rfc_7953_1.ics",
    "collective-icalendar/availabilities/rfc_7953_2.ics",
    "collective-icalendar/calendars/rfc_7953_3.ics",
    "collective-icalendar/events/rfc_9074_example_1.ics",
    "collective-icalendar/events/rfc_9074_example_2.ics",
    "collective-icalendar/events/rfc_9074_example_3.ics",
    "collective-icalendar/events/rfc_9074_example_4.ics",
    "collective-icalendar/events/rfc_9074_example_proximity.ics",
    // --- issue #27 bucket 1: bare component excerpts, no VCALENDAR wrapper
    // ---
    "collective-icalendar/alarms/example.ics",
    "collective-icalendar/alarms/rfc_5545_absolute_alarm_example.ics",
    "collective-icalendar/alarms/rfc_5545_end.ics",
    "collective-icalendar/alarms/start_date.ics",
    "collective-icalendar/calendars/\
     issue_178_component_with_invalid_name_represented.ics",
    "collective-icalendar/calendars/issue_178_custom_component_contains_other.\
     ics",
    "collective-icalendar/calendars/issue_82_expected_output.ics",
    "collective-icalendar/events/event_with_escaped_character1.ics",
    "collective-icalendar/events/event_with_escaped_character2.ics",
    "collective-icalendar/events/event_with_escaped_character3.ics",
    "collective-icalendar/events/event_with_escaped_character4.ics",
    "collective-icalendar/events/event_with_escaped_characters.ics",
    "collective-icalendar/events/event_with_recurrence.ics",
    "collective-icalendar/events/\
     event_with_recurrence_exdates_on_different_lines.ics",
    "collective-icalendar/events/event_with_rsvp.ics",
    "collective-icalendar/events/event_with_unicode_fields.ics",
    "collective-icalendar/events/event_with_unicode_organizer.ics",
    "collective-icalendar/events/\
     issue_100_transformed_doctests_into_unittests.ics",
    "collective-icalendar/events/\
     issue_101_icalendar_chokes_on_umlauts_in_organizer.ics",
    "collective-icalendar/events/issue_104_mark_events_broken.ics",
    "collective-icalendar/events/issue_112_missing_tzinfo_on_exdate.ics",
    "collective-icalendar/events/issue_156_RDATE_with_PERIOD.ics",
    "collective-icalendar/events/issue_156_RDATE_with_PERIOD_list.ics",
    "collective-icalendar/events/issue_157_removes_trailing_semicolon.ics",
    "collective-icalendar/events/issue_184_broken_representation_of_period.ics",
    "collective-icalendar/events/issue_355_url_escaping.ics",
    "collective-icalendar/events/issue_355_url_escaping_2.ics",
    "collective-icalendar/events/issue_355_url_escaping_empty_param.ics",
    "collective-icalendar/events/issue_464_invalid_rdate.ics",
    "collective-icalendar/events/issue_53_description_parsed_properly.ics",
    "collective-icalendar/events/issue_64_event_with_ascii_summary.ics",
    "collective-icalendar/events/issue_64_event_with_non_ascii_summary.ics",
    "collective-icalendar/events/issue_70_rrule_causes_attribute_error.ics",
    "collective-icalendar/events/issue_82_expected_output.ics",
    "collective-icalendar/events/rfc_7265_example_4.ics",
    "collective-icalendar/events/rfc_7265_request_status.ics",
    "collective-icalendar/freebusy/example.ics",
    "collective-icalendar/journals/example.ics",
    "collective-icalendar/timezones/issue_237_brazilia_standard.ics",
    "collective-icalendar/timezones/issue_53_tzid_parsed_properly.ics",
    "collective-icalendar/timezones/\
     issue_55_parse_error_on_utc_offset_with_seconds.ics",
    "collective-icalendar/timezones/pacific_fiji.ics",
    "collective-icalendar/todos/example.ics",
    "libical/8.ics",
    "libical/issue339.ics",
    "libical/overlaps.ics",
    "libical/recur-errors.ics",
    "libical/spanlist.ics",
];

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let fixtures_root = Path::new(&manifest_dir).join("tests/fixtures");
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("fixture_tests.rs");

    let mut out = String::new();

    for dir in ["rfc5545", "rrule", "libical", "collective-icalendar"] {
        let root = fixtures_root.join(dir);
        let mut files = Vec::new();
        collect_files(&root, "ics", &mut files);
        files.sort();
        for file in files {
            let rel_to_fixtures = file
                .strip_prefix(&fixtures_root)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            if MALFORMED_FIXTURES.contains(&rel_to_fixtures.as_str())
                || OUT_OF_SCOPE_FIXTURES.contains(&rel_to_fixtures.as_str())
            {
                continue;
            }
            emit_test(&mut out, &manifest_dir, dir, &root, &file, true);
        }
    }

    let fuzz_root = fixtures_root.join("libical-fuzz-corpus");
    let mut fuzz_files = Vec::new();
    collect_files_excluding(&fuzz_root, "txt", &mut fuzz_files);
    fuzz_files.sort();
    for file in fuzz_files {
        emit_test(
            &mut out,
            &manifest_dir,
            "libical_fuzz_corpus",
            &fuzz_root,
            &file,
            false,
        );
    }

    fs::write(&dest, out).expect("failed to write generated fixture tests");
    println!("cargo:rerun-if-changed=tests/fixtures");
}

/// Recursively collects files under `root` whose extension matches `ext`
/// (case-sensitive, no leading dot).
fn collect_files(root: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, ext, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

/// Recursively collects files under `root` whose extension is anything
/// other than `ext`.
fn collect_files_excluding(root: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_excluding(&path, ext, out);
        } else if path.extension().and_then(|e| e.to_str()) != Some(ext) {
            out.push(path);
        }
    }
}

/// Turns anything that isn't `[A-Za-z0-9_]` into `_` so a file path can be
/// used as a Rust identifier.
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn emit_test(
    out: &mut String,
    manifest_dir: &str,
    group: &str,
    root: &Path,
    file: &Path,
    assert_ok: bool,
) {
    let rel_to_manifest = file
        .strip_prefix(manifest_dir)
        .unwrap()
        .to_str()
        .unwrap()
        .replace('\\', "/");
    let rel_to_root = file
        .strip_prefix(root)
        .unwrap()
        .to_str()
        .unwrap()
        .replace('\\', "/");
    let test_name =
        format!("fixture_{}_{}", sanitize(group), sanitize(&rel_to_root));

    if assert_ok {
        out.push_str(&format!(
            r#"
#[test]
fn {test_name}() {{
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/{rel_to_manifest}");
    let bytes = std::fs::read(path).expect("fixture should be readable");
    if let Err(e) = icalendar::Calendar::parse(&bytes) {{
        panic!("failed to parse {rel_to_manifest}: {{e}}");
    }}
}}
"#
        ));
    } else {
        out.push_str(&format!(
            r#"
#[test]
fn {test_name}() {{
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/{rel_to_manifest}");
    let bytes = std::fs::read(path).expect("fixture should be readable");
    // Fuzz-corpus input: only required not to panic. A parse error is a
    // perfectly fine outcome for adversarial/malformed bytes.
    let _ = icalendar::Calendar::parse(&bytes);
}}
"#
        ));
    }
}
