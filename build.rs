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

use std::{
    env, fs,
    path::{Path, PathBuf},
};

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
