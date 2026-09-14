//! Fixture-driven parser tests (issue #1). Each `#[test]` here is generated
//! by `build.rs` from a file under `tests/fixtures/` — see that file's docs
//! for how the two fixture families (real-world/spec calendars vs. the
//! fuzz corpus) are treated differently.

include!(concat!(env!("OUT_DIR"), "/fixture_tests.rs"));
