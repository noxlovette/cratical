# icalendar

A Rust iCalendar (RFC 5545) library.

## Style conventions

If you see a tuple struct like the Uid, NEVER EVER allow it to have its inner type to be PUBLIC

## Rule enforcement

Components MUST NOT USE ANYTHING OTHER THAN PROPERTIES

Properties MUST have a value and a params struct fields. The value field MUST contain only values from value.rs. Params must only contain values from params.rs, no exceptions

The `vcard` module (behind `rfc-6350`/`rfc-2426`) is a module of its own, not part of the iCalendar model. The rules above apply to it with "the module's own" files: vCard properties hold only values from `src/vcard/values.rs` and params from `src/vcard/params.rs`, never iCalendar-specific types from the top-level `values.rs`/`params.rs`. The format-agnostic primitives (`Text`, `Uri`, `Binary`, `Boolean`, `Integer`, `Float`, `UtcOffset`, `MediaType`) may be reused from the top-level `values.rs` and re-exposed through `vcard/values.rs`, so vCard properties still reference values from one place. Same-named types with different semantics (`ValueDataType`, `Uid`, date/time) are defined separately in `vcard/`, never shared.

## Parser & builder architecture

The parser (`src/ast/parser.rs`) is recursive-descent, single-token lookahead.
A few conventions came out of building it that future work on it MUST follow:

- `Property::parse(name, remainder)` is the single place that maps a property
  keyword to its typed `Property` variant (backed by `PROPERTY_DISPATCH`, a
  `phf::Map`). An unrecognized name isn't an error — it falls back to
  `Xprop`/`Iana`.
- Each component builder implements `PropertyIngest`
  (`fn ingest(&mut self, p: Property) -> ParseResult<()>`) and owns the
  decision of what properties are legal for _it_. Neither the parser nor
  `Component::ingest` inspects a property to decide legality — they just
  parse a `Property` and hand it to the builder, which matches it against
  its own RFC-defined property set and errors on anything else.
- `Component` (wrapping `EventBuilder`/`TodoBuilder`/`JournalBuilder`/
  `FreeBusyBuilder`/`TimezoneBuilder`) models only what RFC 5545 allows
  directly under `VCALENDAR`. Never add a variant for something that can
  only appear nested inside another component (`VALARM`, `STANDARD`,
  `DAYLIGHT`) — that makes an illegal state representable. A sub-component
  gets its own builder type instead, referenced only as a field on whichever
  builder(s) may legally contain it (e.g. `EventBuilder.alarms:
Vec<AlarmBuilder>`, `TimezoneBuilder.standardc`/`daylightc:
Vec<TzPropBuilder>`).
- A sub-component's own `Parser` method (`alarm()`, `tz_observance()`) is a
  sibling to `component()`, not a recursive call into it — each nested
  grammar production has its own alphabet of legal properties and its own
  builder type, so forcing them through one polymorphic function isn't
  worth it. When `component()`'s loop hits a nested `BEGIN`, it dispatches
  by peeking the sub-component's own name (same name-based dispatch as
  `Property::parse` and `component()`'s own top-level match). Whether that
  name is legal _under this particular parent_ is then decided by the
  parent's own `ingest_*` method (e.g. `Component::ingest_alarm` rejects a
  `VALARM` under anything but `VEVENT`/`VTODO`), not by the parser.
- All validation happens at `build()`, not during ingest. Ingest only
  catches what's local to a single property as it arrives — a singleton
  property (`DTSTART`, `UID`, ...) occurring twice (see `set_once`).
  Everything that needs the full picture — mutual exclusion between two
  properties (`DTEND`/`DURATION`), a property requiring another to also be
  present, a component needing at least one of some sub-component
  (`VTIMEZONE` needs ≥1 `STANDARD`/`DAYLIGHT`) — is deferred to `build()`,
  which runs once ingest has seen everything.
- A cross-field check needs to read another property's already-parsed
  value (e.g. `RRULE`'s `UNTIL` vs. the component's `DTSTART`; `ACTION`'s
  kind vs. what `VALARM` requires). Properties keep `value`/`params`
  private per the rule enforcement section above, so the check goes through
  a small `pub(crate)` accessor method on the property type (`RRule::recur`,
  `DateTimeStart::value`, `Action::kind`, ...) — never by making the field
  itself `pub`/`pub(crate)`. The accessor exposes exactly the one thing the
  check needs, so the property can't be constructed or mutated around its
  own invariants from elsewhere in the crate.
- A check that spans more than one _component_ (e.g. a `TZID` parameter
  used on a `DTSTART` matching some `VTIMEZONE`'s `TZID` elsewhere in the
  same object; `UID`/`RECURRENCE-ID` uniqueness across components) can't
  live in any single builder's `build()` — it doesn't have the other
  components yet. These run once in `CalendarBuilder::build()`, as a free
  function taking the already-built `&[calendar::Component]` slice, called
  right after every individual component has been built (so per-component
  rules have already run) and before `Calendar` is assembled. See
  `validate_timezones`/`validate_no_duplicate_uid` in `src/ast.rs`.

## Testing philosophy

Tests are written test-driven: they assert what the code's behavior MUST be per RFC 5545 (or the type's documented contract), not what the current implementation happens to do. Never write a test by running the code and copying its output into the assertion — that encodes bugs as spec. If a test fails against current code, the test is correct and the code is wrong; fix the code, don't loosen the test.

### `tests/fixtures/` is untouchable

Every file under `tests/fixtures/` is ground truth, not a knob to turn when a
test fails. This includes the hand-authored `rfc5545/`/`rrule/` fixtures as
much as the imported `collective-icalendar/`/`libical/`/`libical-fuzz-corpus/`
corpora. **Never edit, "fix", regenerate, delete, or add fields to a fixture
file to make a failing test pass** — that's the exact same violation as
loosening an assertion, just laundered through a `.ics` file instead of a
`#[test]` body. If a fixture fails to parse (or parses wrong), the crate is
wrong and the fixture stays exactly as it is; fix `src/`, not
`tests/fixtures/`. This applies even when the fixture looks like it could
plausibly be "improved" (e.g. adding a `VTIMEZONE` block, adding a missing
required property) — that instinct is backwards here.

The one exception: adding genuinely new files to the corpus (e.g. importing
another upstream test suite) is fine. Editing or removing an existing one is
not.

## Documentation conventions

Every public type and property struct/enum must have a doc comment that:

1. Includes the verbatim **Description** paragraph(s) from RFC 5545 (no "Description:" label).
2. Includes an **Example** section with a `>` blockquote showing the iCalendar wire format.
3. Ends with a RFC section link in the form:
   `[Section X.X.X](https://datatracker.ietf.org/doc/html/rfc5545#section-X.X.X)`

See `src/values.rs` and `src/properties/calendar.rs` for established examples of this pattern.

The authoritative reference is [RFC 5545](https://datatracker.ietf.org/doc/html/rfc5545).
