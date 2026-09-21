use super::token::Token;
#[cfg(feature = "rfc-9074")]
use crate::ast::VLocationParseBuilder;
#[cfg(feature = "rfc-7953")]
use crate::ast::{AvailabilityParseBuilder, AvailableParseBuilder};
use crate::{
    Calendar,
    ast::{
        AlarmParseBuilder, CalendarParseBuilder, Component, ComponentError,
        EventParseBuilder, FreeBusyParseBuilder, JournalParseBuilder, Property,
        PropertyIngest, TimezoneParseBuilder, TodoParseBuilder,
        TzObservanceKind, TzPropParseBuilder, UnknownComponentBuilder,
        token::TokenType,
    },
    params::ParamError,
    properties::{ParameterError, PropertyError},
    values::ValueError,
};
use TokenType::*;
use std::str::Utf8Error;
use thiserror::Error;

/// parses Tokens into valid iCal Formal Grammar
///
/// The grammar is recursive-descent, top-down, single-token lookahead:
/// [`Self::calendar`] recurses into [`Self::component`] on `BEGIN`, which
/// recurses further for nested components (`VALARM` inside `VEVENT`, etc.
/// — not yet wired up). There's no sub-line grammar left to descend into
/// here — a property line arrives from the lexer as one opaque
/// [`TokenType::Property`] token, and [`Property::parse`] (backed by a
/// name -> parser dispatch table, see `crate::ast::PROPERTY_DISPATCH`)
/// hands back a fully-parsed [`Property`] in one step.
#[derive(Default, Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// creates a new parser
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            ..Default::default()
        }
    }

    /// main entry point. Parses one `BEGIN:VCALENDAR ... END:VCALENDAR`
    /// iCalendar object (RFC 5545 §3.4/§3.6) and builds it, running every
    /// cross-field validation deferred to `build()` (see `crate::ast`'s
    /// module docs). A stream containing more than one `icalobject` can be
    /// parsed by calling this repeatedly — it consumes exactly one
    /// `VCALENDAR` and leaves the parser positioned right after it.
    pub fn calendar(&mut self) -> ParseResult<Calendar> {
        if self
            .consume(Begin, "expected calendar to start with BEGIN")?
            .literal()
            != b"VCALENDAR"
        {
            return Err(ParseError::UnknownComponent);
        }
        self.consume(Crlf, "expected crlf after BEGIN")?;

        let mut cal = CalendarParseBuilder::new();

        // Calendar properties (§3.7) precede any component.
        while !self.check(Begin)? {
            let prop =
                self.consume(Property, "expected a calendar property")?;
            let property = Property::parse(prop.lexeme(), prop.literal())?;
            self.consume(Crlf, "expected crlf after property")?;

            match property {
                Property::ProductIdentifier(p) => cal.prodid = Some(p),
                Property::Version(v) => cal.version = Some(v),
                Property::Method(m) => cal.method = Some(m),
                Property::CalendarScale(c) => cal.calscale = Some(c),
                // RFC 7986 §5 new/extended `VCALENDAR`-level properties —
                // core, not feature-gated (see `crate::ast`'s
                // `property_dispatch_map!` doc comment).
                Property::Uid(v) => {
                    crate::ast::set_once(&mut cal.uid, v, "UID")?
                }
                Property::LastModified(v) => {
                    crate::ast::set_once(&mut cal.last_mod, v, "LAST-MODIFIED")?
                }
                Property::UniformResourceLocator(v) => {
                    crate::ast::set_once(&mut cal.url, v, "URL")?
                }
                Property::RefreshInterval(v) => crate::ast::set_once(
                    &mut cal.refresh_interval,
                    v,
                    "REFRESH-INTERVAL",
                )?,
                Property::Source(v) => {
                    crate::ast::set_once(&mut cal.source, v, "SOURCE")?
                }
                Property::Color(v) => {
                    crate::ast::set_once(&mut cal.color, v, "COLOR")?
                }
                Property::Name(v) => cal.name.push(v),
                Property::Description(v) => cal.description.push(v),
                Property::Categories(v) => cal.categories.push(v),
                Property::Image(v) => cal.image.push(v),
                Property::Xprop(x) => cal.xprop.push(x),
                Property::Iana(i) => cal.iana.push(i),
                _ => return Err(PropertyError::UnexpectedProperty.into()),
            }
        }

        while self.check(Begin)? {
            cal.components.push(self.component()?);
        }

        if self
            .consume(End, "expected calendar to end with END")?
            .literal()
            != b"VCALENDAR"
        {
            return Err(ParseError::MismatchedEnd);
        }
        // Real-world producers routinely omit the final line terminator
        // after the outermost `END:VCALENDAR` (many editors/exporters trim
        // a trailing newline). Every other CRLF in the grammar is still
        // mandatory — there's always more content after it — but this one
        // is only ever followed by EOF or, in a multi-object stream (see
        // `Calendar::parse_stream`), a CRLF followed by the next object's
        // `BEGIN`. The latter case still has a real CRLF token to consume;
        // only true EOF (nothing left at all) gets a pass.
        if !self.is_at_end()? {
            self.consume(Crlf, "expected crlf after END")?;
        }

        Ok(cal.build()?)
    }

    /// parses one `BEGIN:<name> ... END:<name>` component, routing its
    /// property lines into the matching builder
    fn component(&mut self) -> ParseResult<Component> {
        let begin =
            self.consume(Begin, "expected component to start with BEGIN")?;
        let name = begin.literal().to_vec();

        let mut component: Component = match name.as_slice() {
            b"VEVENT" => EventParseBuilder::new().into(),
            b"VTODO" => TodoParseBuilder::new().into(),
            b"VJOURNAL" => JournalParseBuilder::new().into(),
            b"VFREEBUSY" => FreeBusyParseBuilder::new().into(),
            b"VTIMEZONE" => TimezoneParseBuilder::new().into(),
            #[cfg(feature = "rfc-7953")]
            b"VAVAILABILITY" => AvailabilityParseBuilder::new().into(),
            // An unrecognized `iana-comp`/`x-comp` (RFC 5545 §3.6). Not an
            // error — the RFC requires applications to ignore a component
            // type they don't recognize, and discourages silently dropping
            // it. There's no typed builder for it, so its body is parsed
            // opaquely by `unknown_component_body` instead of the loop
            // below, and returned immediately.
            _ => {
                self.consume(Crlf, "expected crlf after BEGIN")?;
                return Ok(self.unknown_component_body(name)?.into());
            }
        };
        self.consume(Crlf, "expected crlf after BEGIN")?;

        while !self.check(End)? {
            if self.check(Begin)? {
                // Dispatch on the sub-component's own name, same as the
                // top-level `match` above — legality of nesting it *here*
                // is then decided by `component`'s own `ingest_*` (a
                // `VALARM` under `VJOURNAL`, say, parses fine below and is
                // rejected by `ingest_alarm`).
                match self.peek()?.literal() {
                    b"VALARM" => {
                        let alarm = self.alarm()?;
                        component.ingest_alarm(alarm)?;
                    }
                    b"STANDARD" | b"DAYLIGHT" => {
                        let (kind, tz_prop) = self.tz_observance()?;
                        component.ingest_tz_observance(kind, tz_prop)?;
                    }
                    #[cfg(feature = "rfc-7953")]
                    b"AVAILABLE" => {
                        let available = self.available()?;
                        component.ingest_available(available)?;
                    }
                    _ => return Err(ParseError::UnknownComponent),
                }
            } else {
                let prop =
                    self.consume(Property, "expected a property line")?;
                let property = Property::parse(prop.lexeme(), prop.literal())?;
                self.consume(Crlf, "expected crlf after property")?;
                component.ingest(property)?;
            }
        }

        let end = self.consume(End, "expected component to end with END")?;
        if end.literal() != name.as_slice() {
            return Err(ParseError::MismatchedEnd);
        }
        self.consume(Crlf, "expected crlf after END")?;

        Ok(component)
    }

    /// parses one `BEGIN:VALARM ... END:VALARM` sub-component (RFC 5545
    /// §3.6.6). This is deliberately not a recursive call into
    /// [`Self::component`]: `VALARM` is a distinct grammar production with
    /// its own alphabet of legal properties and its own builder type
    /// ([`AlarmParseBuilder`], not [`Component`]). Under the `rfc-9074`
    /// feature, a `VALARM` can itself nest `VLOCATION` sub-components (RFC
    /// 9073 §7.2, via RFC 9074 §8's proximity extension) — same
    /// dispatch-by-name shape as [`Self::component`]'s own nested-`BEGIN`
    /// handling, just with a single legal name instead of several.
    fn alarm(&mut self) -> ParseResult<AlarmParseBuilder> {
        let begin =
            self.consume(Begin, "expected sub-component to start with BEGIN")?;
        if begin.literal() != b"VALARM" {
            return Err(ParseError::UnknownComponent);
        }
        self.consume(Crlf, "expected crlf after BEGIN")?;

        let mut alarm = AlarmParseBuilder::new();
        while !self.check(End)? {
            if self.check(Begin)? {
                match self.peek()?.literal() {
                    #[cfg(feature = "rfc-9074")]
                    b"VLOCATION" => {
                        let location = self.location()?;
                        alarm.locations.push(location);
                    }
                    _ => return Err(ParseError::UnknownComponent),
                }
            } else {
                let prop =
                    self.consume(Property, "expected a property line")?;
                let property = Property::parse(prop.lexeme(), prop.literal())?;
                self.consume(Crlf, "expected crlf after property")?;
                alarm.ingest(property)?;
            }
        }

        let end =
            self.consume(End, "expected sub-component to end with END")?;
        if end.literal() != b"VALARM" {
            return Err(ParseError::MismatchedEnd);
        }
        self.consume(Crlf, "expected crlf after END")?;

        Ok(alarm)
    }

    /// parses one `BEGIN:VLOCATION ... END:VLOCATION` sub-component (RFC
    /// 9073 §7.2), nested only inside `VALARM` — same shape as
    /// [`Self::alarm`]/[`Self::available`]: its own grammar production, its
    /// own builder type ([`VLocationParseBuilder`]), no further nesting.
    #[cfg(feature = "rfc-9074")]
    fn location(&mut self) -> ParseResult<VLocationParseBuilder> {
        let begin =
            self.consume(Begin, "expected sub-component to start with BEGIN")?;
        if begin.literal() != b"VLOCATION" {
            return Err(ParseError::UnknownComponent);
        }
        self.consume(Crlf, "expected crlf after BEGIN")?;

        let mut location = VLocationParseBuilder::new();
        while !self.check(End)? {
            let prop = self.consume(Property, "expected a property line")?;
            let property = Property::parse(prop.lexeme(), prop.literal())?;
            self.consume(Crlf, "expected crlf after property")?;
            location.ingest(property)?;
        }

        let end =
            self.consume(End, "expected sub-component to end with END")?;
        if end.literal() != b"VLOCATION" {
            return Err(ParseError::MismatchedEnd);
        }
        self.consume(Crlf, "expected crlf after END")?;

        Ok(location)
    }

    /// parses one `BEGIN:AVAILABLE ... END:AVAILABLE` sub-component (RFC
    /// 7953 §3.1), nested only inside `VAVAILABILITY` — same shape as
    /// [`Self::alarm`]: its own grammar production, its own builder type
    /// ([`AvailableParseBuilder`], not [`Component`]), no further nesting.
    #[cfg(feature = "rfc-7953")]
    fn available(&mut self) -> ParseResult<AvailableParseBuilder> {
        let begin =
            self.consume(Begin, "expected sub-component to start with BEGIN")?;
        if begin.literal() != b"AVAILABLE" {
            return Err(ParseError::UnknownComponent);
        }
        self.consume(Crlf, "expected crlf after BEGIN")?;

        let mut available = AvailableParseBuilder::new();
        while !self.check(End)? {
            let prop = self.consume(Property, "expected a property line")?;
            let property = Property::parse(prop.lexeme(), prop.literal())?;
            self.consume(Crlf, "expected crlf after property")?;
            available.ingest(property)?;
        }

        let end =
            self.consume(End, "expected sub-component to end with END")?;
        if end.literal() != b"AVAILABLE" {
            return Err(ParseError::MismatchedEnd);
        }
        self.consume(Crlf, "expected crlf after END")?;

        Ok(available)
    }

    /// parses one `BEGIN:STANDARD ... END:STANDARD` or `BEGIN:DAYLIGHT ...
    /// END:DAYLIGHT` sub-component (RFC 5545 §3.6.5) — structurally the
    /// same shape as [`Self::alarm`]: its own grammar production
    /// (`STANDARD`/`DAYLIGHT` share one property alphabet, `tzprop`), its
    /// own builder type ([`TzPropParseBuilder`]), no further nesting.
    fn tz_observance(
        &mut self,
    ) -> ParseResult<(TzObservanceKind, TzPropParseBuilder)> {
        let begin =
            self.consume(Begin, "expected sub-component to start with BEGIN")?;
        let kind = match begin.literal() {
            b"STANDARD" => TzObservanceKind::Standard,
            b"DAYLIGHT" => TzObservanceKind::Daylight,
            _ => return Err(ParseError::UnknownComponent),
        };
        let name = begin.literal().to_vec();
        self.consume(Crlf, "expected crlf after BEGIN")?;

        let mut tz_prop = TzPropParseBuilder::new();
        while !self.check(End)? {
            let prop = self.consume(Property, "expected a property line")?;
            let property = Property::parse(prop.lexeme(), prop.literal())?;
            self.consume(Crlf, "expected crlf after property")?;
            tz_prop.ingest(property)?;
        }

        let end =
            self.consume(End, "expected sub-component to end with END")?;
        if end.literal() != name.as_slice() {
            return Err(ParseError::MismatchedEnd);
        }
        self.consume(Crlf, "expected crlf after END")?;

        Ok((kind, tz_prop))
    }

    /// parses the body of an unrecognized top-level component (RFC 5545
    /// §3.6 `iana-comp`/`x-comp`) once its `BEGIN:<name>` line has already
    /// been consumed by [`Self::component`]. Unlike [`Self::alarm`]/
    /// [`Self::tz_observance`], there's no typed builder to route content
    /// into — an unrecognized component's own alphabet of legal properties
    /// isn't something this crate can know — so every content line is kept
    /// verbatim, and any nested `BEGIN` (known or unknown) is captured the
    /// same opaque way, recursively.
    fn unknown_component_body(
        &mut self,
        name: Vec<u8>,
    ) -> ParseResult<UnknownComponentBuilder> {
        let mut builder = UnknownComponentBuilder::new(name.clone());

        while !self.check(End)? {
            if self.check(Begin)? {
                let nested = self
                    .consume(Begin, "expected component to start with BEGIN")?;
                let nested_name = nested.literal().to_vec();
                self.consume(Crlf, "expected crlf after BEGIN")?;
                builder
                    .components
                    .push(self.unknown_component_body(nested_name)?);
            } else {
                let prop =
                    self.consume(Property, "expected a property line")?;
                let mut line = prop.lexeme().to_vec();
                line.extend_from_slice(prop.literal());
                builder.lines.push(line);
                self.consume(Crlf, "expected crlf after property")?;
            }
        }

        let end = self.consume(End, "expected component to end with END")?;
        if end.literal() != name.as_slice() {
            return Err(ParseError::MismatchedEnd);
        }
        self.consume(Crlf, "expected crlf after END")?;

        Ok(builder)
    }

    /// returns true if the next token corresponds to the one passed to the
    /// function. false if we have reached the end of the vector
    fn check(&mut self, t: TokenType) -> ParseResult<bool> {
        Ok(self.peek()?.token_type() == t)
    }

    /// consumes a token that a certain grammar rule expects
    fn consume(
        &mut self,
        tt: TokenType,
        msg: &'static str,
    ) -> ParseResult<&Token> {
        if self.check(tt)? {
            self.next()
        } else {
            Err(self.error(msg))
        }
    }

    /// error convenience wrapper
    fn error(&self, msg: &'static str) -> ParseError {
        match self.peek() {
            Ok(t) => ParseError::UnexpectedToken {
                line: t.line(),
                msg,
                lexeme: t.lexeme_as_string(),
            },
            Err(e) => e,
        }
    }

    /// checks whether we are at the end of the tokens list
    pub(crate) fn is_at_end(&self) -> ParseResult<bool> {
        Ok(self.peek()?.token_type() == Eof)
    }

    /// returns the next token. doesn't advance the parser
    fn peek(&self) -> ParseResult<&Token> {
        self.tokens
            .get(self.current)
            .ok_or(ParseError::UnexpectedEof)
    }

    /// advances the parser and returns the next token
    fn next(&mut self) -> ParseResult<&Token> {
        if !self.is_at_end()? {
            self.current += 1;
        }
        self.prev()
    }

    /// gets the latest token
    fn prev(&self) -> ParseResult<&Token> {
        self.tokens
            .get(self.current - 1)
            .ok_or(ParseError::EmptyTokenList)
    }
}

/// Convenience wrapper for [ParseError]
pub(crate) type ParseResult<T> = Result<T, ParseError>;

#[derive(Error, Debug)]
/// Parsing error
pub enum ParseError {
    /// Parameter Parsing Error
    #[error("Parameter parsing failed. Expected {expected}, got {received:?}")]
    Parameter {
        /// What the parameter is supposed to be
        expected: String,
        /// What we actually received
        received: Option<String>,
    },

    #[error("unknown component")]
    UnknownComponent,

    #[error(transparent)]
    Property(#[from] PropertyError),

    /// A recognized sub-component (`VALARM`, `STANDARD`, `DAYLIGHT`) turned
    /// up somewhere RFC 5545 doesn't allow it to be nested.
    #[error("{0} is not a legal sub-component here")]
    UnexpectedComponent(&'static str),

    #[error("component's END name doesn't match its BEGIN name")]
    MismatchedEnd,

    /// Encoding error
    #[error(transparent)]
    Utf(#[from] Utf8Error),

    #[error("Unexpected EOF")]
    UnexpectedEof,

    #[error("Unexpected Token at line {line} ({lexeme}): {msg}")]
    UnexpectedToken {
        line: usize,
        msg: &'static str,
        lexeme: String,
    },

    #[error("Ran prev on an empty token list")]
    EmptyTokenList,

    #[error(transparent)]
    Component(#[from] ComponentError),

    /// A single property parameter (`ALTREP`, `LANGUAGE`, ...) failed to
    /// parse into its typed representation. See [`ParamError`].
    #[error(transparent)]
    Param(#[from] ParamError),

    /// The `*(";" param)` parameter list of a content line failed to parse
    /// — a malformed `NAME=VALUE` segment, or one of its params. See
    /// [`ParameterError`].
    #[error(transparent)]
    Parameters(#[from] ParameterError),

    /// A property's value failed to parse into its typed representation.
    /// See [`ValueError`].
    #[error(transparent)]
    Value(#[from] ValueError),
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::lexer::Lexer;

    fn parse(src: &[u8]) -> ParseResult<Calendar> {
        let tokens = Lexer::new(src).scan().unwrap();
        Parser::new(tokens).calendar()
    }

    const MINIMAL_EVENT: &[u8] = b"BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:123@example.com\r\nDTSTAMP:19970901T130000Z\r\nDTSTART:19970903T163000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    #[test]
    fn parses_a_minimal_calendar_with_one_event() {
        let cal = parse(MINIMAL_EVENT).unwrap();
        assert_eq!(cal.components.len(), 1);
    }

    #[test]
    fn rejects_mismatched_begin_end() {
        let src = b"BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:123@example.com\r\nDTSTAMP:19970901T130000Z\r\nDTSTART:19970903T163000Z\r\nEND:VEVENT\r\nEND:VJOURNAL\r\n";
        assert!(matches!(parse(src), Err(ParseError::MismatchedEnd)));
    }

    #[test]
    fn captures_an_unrecognized_top_level_component_instead_of_erroring() {
        // RFC 5545 §3.6: applications MUST ignore an iana-comp/x-comp they
        // don't recognize, and SHOULD NOT silently drop it — it must not
        // poison parsing of the rest of the VCALENDAR (issue #13).
        let src = b"BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\nBEGIN:X-FOO\r\nUID:1234\r\nEND:X-FOO\r\nBEGIN:VEVENT\r\nUID:123@example.com\r\nDTSTAMP:19970901T130000Z\r\nDTSTART:19970903T163000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let cal = parse(src).unwrap();
        assert_eq!(cal.components.len(), 2);

        let crate::calendar::Component::Unknown(unknown) = &cal.components[0]
        else {
            panic!("expected the first component to be Unknown");
        };
        assert_eq!(unknown.name(), "X-FOO");
        assert_eq!(unknown.lines(), ["UID:1234"]);
        assert!(unknown.components().is_empty());

        assert!(matches!(
            cal.components[1],
            crate::calendar::Component::Event(_)
        ));
    }

    #[test]
    fn unrecognized_top_level_component_preserves_components_nested_inside_it()
    {
        let src = b"BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\nBEGIN:X-OUTER\r\nBEGIN:X-INNER\r\nX-PROP:value\r\nEND:X-INNER\r\nEND:X-OUTER\r\nEND:VCALENDAR\r\n";
        let cal = parse(src).unwrap();
        assert_eq!(cal.components.len(), 1);

        let crate::calendar::Component::Unknown(outer) = &cal.components[0]
        else {
            panic!("expected the component to be Unknown");
        };
        assert_eq!(outer.name(), "X-OUTER");
        assert!(outer.lines().is_empty());
        assert_eq!(outer.components().len(), 1);
        assert_eq!(outer.components()[0].name(), "X-INNER");
        assert_eq!(outer.components()[0].lines(), ["X-PROP:value"]);
    }

    #[test]
    fn rejects_mismatched_begin_end_inside_an_unrecognized_component() {
        let src = b"BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\nBEGIN:X-FOO\r\nEND:X-BAR\r\nEND:VCALENDAR\r\n";
        assert!(matches!(parse(src), Err(ParseError::MismatchedEnd)));
    }

    #[test]
    fn rejects_a_begin_that_is_not_vcalendar() {
        let src = b"BEGIN:VEVENT\r\nEND:VEVENT\r\n";
        assert!(matches!(parse(src), Err(ParseError::UnknownComponent)));
    }

    #[test]
    fn requires_at_least_one_component() {
        let src = b"BEGIN:VCALENDAR\r\nPRODID:-//example//EN\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";
        assert!(parse(src).is_err());
    }

    #[test]
    fn parses_two_sequential_calendar_objects_from_one_stream() {
        let mut src = MINIMAL_EVENT.to_vec();
        src.extend_from_slice(MINIMAL_EVENT);
        let tokens = Lexer::new(&src).scan().unwrap();
        let mut parser = Parser::new(tokens);
        assert!(parser.calendar().is_ok());
        assert!(parser.calendar().is_ok());
    }
}
