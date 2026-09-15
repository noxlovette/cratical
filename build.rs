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
//! A handful of files under `collective-icalendar/calendars/` are themselves
//! deliberately malformed (see issue #4) rather than real-world-valid — they
//! are excluded from the blanket "must parse successfully" generation below
//! (`MALFORMED_FIXTURES`) and instead get individual, specific assertions in
//! `tests/malformed_input.rs`.
//!
//! Another handful cover components/properties this crate has deliberately
//! decided not to implement (`VLOCATION`, the RFC 9074 `VALARM`
//! extensions — see issue #7 and `src/components.rs`'s module doc), or are
//! excerpts too incomplete to form a valid `icalobject` on their own (the
//! three RFC 7953 `VAVAILABILITY` fixtures — bare excerpts with no
//! `VCALENDAR` wrapper, or missing `PRODID`/`VERSION`; `VAVAILABILITY`
//! itself is implemented under the `rfc_7953` feature, see issue #16) —
//! excluded the same way (`OUT_OF_SCOPE_FIXTURES`), with dedicated
//! assertions in `tests/out_of_scope.rs`.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Files under `collective-icalendar/` that are deliberately malformed
/// (real-world-broken exports, RFC violations, fuzzer-found edge cases —
/// see issue #4), not valid calendars that happen to fail. Excluded from the
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
];

/// Files covering RFC 9073 (`VLOCATION`)/RFC 9074 (`VALARM` extensions) —
/// real-world-valid for those RFCs, but this crate deliberately doesn't
/// implement them (see issue #7 and `src/components.rs`'s module doc) —
/// plus the three RFC 7953 (`VAVAILABILITY`) fixtures, which this crate
/// *does* implement (under the `rfc_7953` feature, see issue #16) but are
/// themselves too incomplete to form a valid `icalobject` (bare excerpts
/// with no `VCALENDAR` wrapper, or missing `PRODID`/`VERSION`). Excluded
/// from the blanket "must parse successfully" generation either way; each
/// gets a dedicated assertion in `tests/out_of_scope.rs` instead.
const OUT_OF_SCOPE_FIXTURES: &[&str] = &[
    "collective-icalendar/availabilities/rfc_7953_1.ics",
    "collective-icalendar/availabilities/rfc_7953_2.ics",
    "collective-icalendar/calendars/rfc_7953_3.ics",
    "collective-icalendar/events/rfc_9074_example_1.ics",
    "collective-icalendar/events/rfc_9074_example_2.ics",
    "collective-icalendar/events/rfc_9074_example_3.ics",
    "collective-icalendar/events/rfc_9074_example_4.ics",
    "collective-icalendar/events/rfc_9074_example_proximity.ics",
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
