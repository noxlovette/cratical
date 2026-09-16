# Cratical

> The name is inspired by the libical repo. In our case, it's crate + ical

## RFC coverage

### Core (always compiled in, no feature flag)

- **[RFC 5545](https://datatracker.ietf.org/doc/html/rfc5545)** — the
  iCalendar object model itself: `VCALENDAR` and its `VEVENT`/`VTODO`/
  `VJOURNAL`/`VFREEBUSY`/`VTIMEZONE` components, their properties, parameters,
  and value types.
- **[RFC 6868](https://datatracker.ietf.org/doc/html/rfc6868)** — the
  `^n`/`^'`/`^^` CARET-encoding used to embed newlines and double quotes in a
  parameter value. See `decode_caret` in `src/ast.rs`.
- **[RFC 7986](https://datatracker.ietf.org/doc/html/rfc7986)** — new
  properties (`NAME`, `UID`/`DESCRIPTION`/`LAST-MODIFIED`/`URL`/`CATEGORIES`
  extended onto `VCALENDAR`, `REFRESH-INTERVAL`, `SOURCE`, `COLOR`, `IMAGE`,
  `CONFERENCE`) and the `DISPLAY`/`FEATURE`/`LABEL` parameters they use. This
  *updates* RFC 5545 itself rather than adding a distinct optional component
  or extension the way RFC 7953/9074 do below, so it's treated as core here
  too, not behind a feature flag.

### Feature-gated (opt out by disabling default features)

- **`rfc-7953`** — **[RFC 7953](https://datatracker.ietf.org/doc/html/rfc7953)**:
  the `VAVAILABILITY`/`AVAILABLE` components for publishing free/busy
  availability. Relevant if you're building a scheduling server; skip it for
  plain calendar storage/display.
- **`rfc-9074`** — **[RFC 9074](https://datatracker.ietf.org/doc/html/rfc9074)**:
  `VALARM`'s `UID`/`RELATED-TO`/`ACKNOWLEDGED`/`PROXIMITY` extensions, for
  deduplicating and syncing alarms across devices. Also currently carries the
  one piece of **[RFC 9073](https://datatracker.ietf.org/doc/html/rfc9073)**
  this crate implements — the `VLOCATION` sub-component nested inside
  `VALARM` (RFC 9074 §8's proximity extension) — since that's the only
  context this crate parses it in.
- **`rfc-5546`** — **[RFC 5546](https://datatracker.ietf.org/doc/html/rfc5546)**
  (iTIP): a typed `itip::Method` for the 8 scheduling methods
  (`PUBLISH`/`REQUEST`/`REPLY`/`ADD`/`CANCEL`/`REFRESH`/`COUNTER`/
  `DECLINECOUNTER`), and validation of a `VCALENDAR` against RFC 5546's
  per-method component/property restriction tables whenever `METHOD` names
  one of them. Only checks restrictions verifiable from a single iCalendar
  object — see `src/itip.rs`'s module docs for what's out of scope (anything
  requiring comparison against an earlier message) and why. Relevant if
  you're implementing calendar scheduling (invites, RSVPs); real-world
  calendar exports that stamp `METHOD` without full iTIP conformance (e.g. a
  personal reminder tagged `METHOD:PUBLISH` with no `ORGANIZER`) will be
  rejected once this is enabled — see `tests/rfc_5546.rs`.

### Not yet implemented

These would each be their own opt-in feature (matching the `rfc-7953`/
`rfc-9074` pattern above) if added — none of them are needed for plain
calendar storage/display, which is this crate's current focus:

- **RFC 7529** (`RSCALE`) — non-Gregorian recurrence rules (`RRULE`'s
  `RSCALE=CHINESE` etc.).
- **RFC 9073** (beyond the `VLOCATION`-in-`VALARM` piece above) — structured
  data such as `VRESOURCE` and `PARTICIPANT`, relevant for JSCalendar interop
  or structured event publishing.
- **RFC 9253** — `RELATED-TO`'s extended `RELTYPE` values and the `LINK`
  property, for modeling richer event relationships/dependencies. Today
  these pass straight through this crate's ordinary IANA/`X-`-prefixed
  extension fallback rather than getting typed support.

## Test data attribution

Delivering a correct, dependable parser isn't possible by testing only against
hand-written examples — real `.ics` files produced by actual calendar clients
(Google Calendar, Thunderbird, Etar, khal, and others) are full of edge cases,
quirks, and outright spec violations that synthetic fixtures never surface.

`tests/fixtures/collective-icalendar/` vendors the `.ics` test fixtures from
[`collective/icalendar`](https://github.com/collective/icalendar), the most
widely used iCalendar library on GitHub, whose test suite has accumulated
these real-world files over more than a decade of bug reports and interop
fixes. They're used here under the terms of that project's BSD-style license
(Copyright (c) 2012-2013, Plone Foundation; see
[`LICENSE.rst`](https://github.com/collective/icalendar/blob/main/LICENSE.rst)).

`tests/fixtures/libical/` and `tests/fixtures/libical-fuzz-corpus/` vendor the
curated `.ics` test cases and fuzzer-discovered crash corpus from
[`libical/libical`](https://github.com/libical/libical) (`test-data/`, `4.0`
branch), the reference C implementation of iCalendar. The fuzz corpus in
particular exercises parser-robustness edge cases (malformed/adversarial
input) that no hand-written fixture set would think to construct. These files
are used under the terms libical itself declares for that directory
(`LGPL-2.1-only OR MPL-2.0`; Copyright 1999 Contributors to the Libical
project) — see the `LICENSE.txt`, `COPYING.LESSER.txt`, and `LICENSES/`
copied alongside them in each fixture directory. This license is **narrower
than, and does not change, this crate's own `Apache-2.0` license**: MPL-2.0's
copyleft is file-level (it doesn't extend to unrelated files in the same
repo) and LGPL-2.1's obligations are about linking compiled code into a
program, which plain-text test data never does. To keep the published crate
package unambiguous, `tests/fixtures/` is excluded from what gets shipped to
crates.io (see `Cargo.toml`) — these fixtures exist for this repo's own test
suite, not as part of the distributed library.
