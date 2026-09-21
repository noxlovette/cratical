use super::{ParseError, VCard, VCardBuilder, properties::Property};
use crate::ast::token::{Token, TokenType};
use TokenType::*;

/// Parses the tokens of [`Lexer::vcard`](crate::ast::Lexer::vcard) into
/// vCards.
///
/// Recursive-descent with single-token lookahead, like the iCalendar
/// [`Parser`](crate::ast::parser::Parser), but a sibling of it rather than
/// a reuse: a vCard has its own grammar production ([`Self::vcard`]) with
/// its own property alphabet and rules, and nothing nests inside it.
#[derive(Debug)]
pub(crate) struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub(crate) fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parses one `BEGIN:VCARD ... END:VCARD` object (RFC 6350 §3.3):
    ///
    /// ```text
    /// vcard = "BEGIN:VCARD" CRLF
    ///         "VERSION:4.0" CRLF
    ///         1*contentline
    ///         "END:VCARD" CRLF
    /// ```
    ///
    /// leaving the parser right after it, so it can be called again for
    /// the next object of a `vcard-entity`.
    ///
    /// The `VERSION` rule differs by version: 4.0 requires it right after
    /// `BEGIN:VCARD`, while 3.0 (RFC 2426) only requires it to be present,
    /// anywhere. It can't be enforced while reading properties, because the
    /// version isn't known until `VERSION` is read, so the position is
    /// recorded and checked once the object ends.
    pub(crate) fn vcard(&mut self) -> Result<VCard, ParseError> {
        let begin =
            self.consume(Begin, "expected vCard to start with BEGIN")?;
        if begin.literal() != b"VCARD" {
            return Err(ParseError::NotVCard {
                line: begin.line(),
                name: String::from_utf8_lossy(begin.literal()).into_owned(),
            });
        }
        self.consume(Crlf, "expected crlf after BEGIN")?;

        let mut version = None;
        let mut version_first = false;
        let mut properties = Vec::new();

        while !self.check(End)? {
            if self.is_at_end()? {
                return Err(ParseError::UnexpectedEof);
            }
            if self.check(Begin)? {
                return Err(ParseError::NestedComponent {
                    line: self.peek()?.line(),
                });
            }

            let token = self.consume(Property, "expected a property line")?;
            let property = Property::parse(
                token.group(),
                token.lexeme(),
                token.literal(),
            )?;
            self.consume(Crlf, "expected crlf after property")?;

            match property {
                Property::Version(v) => {
                    if version.is_some() {
                        return Err(ParseError::DuplicateVersion);
                    }
                    version_first = properties.is_empty();
                    version = Some(v);
                }
                other => properties.push(other),
            }
        }

        let end = self.consume(End, "expected vCard to end with END")?;
        if end.literal() != b"VCARD" {
            return Err(ParseError::MismatchedEnd {
                line: end.line(),
                name: String::from_utf8_lossy(end.literal()).into_owned(),
            });
        }
        // The one leniency: the outermost `END:VCARD` may lack its line
        // terminator at EOF. Anywhere else more content follows it, so the
        // CRLF is still mandatory.
        if !self.is_at_end()? {
            self.consume(Crlf, "expected crlf after END")?;
        }

        let version = version.ok_or(ParseError::MissingVersion)?;
        if version.value() == super::values::Version::V4_0 && !version_first {
            return Err(ParseError::VersionNotFirst);
        }

        // What needs the whole card is the builder's to check.
        let mut builder = VCardBuilder::with_version(version);
        for property in properties {
            builder.ingest(property);
        }
        Ok(builder.build()?)
    }

    /// whether the next token is of type `t`
    fn check(&self, t: TokenType) -> Result<bool, ParseError> {
        Ok(self.peek()?.token_type() == t)
    }

    /// consumes a token the grammar expects
    fn consume(
        &mut self,
        tt: TokenType,
        msg: &'static str,
    ) -> Result<&Token, ParseError> {
        if self.check(tt)? {
            self.next()
        } else {
            let t = self.peek()?;
            Err(ParseError::UnexpectedToken {
                line: t.line(),
                msg,
                lexeme: t.lexeme_as_string(),
            })
        }
    }

    pub(crate) fn is_at_end(&self) -> Result<bool, ParseError> {
        Ok(self.peek()?.token_type() == Eof)
    }

    fn peek(&self) -> Result<&Token, ParseError> {
        self.tokens
            .get(self.current)
            .ok_or(ParseError::UnexpectedEof)
    }

    fn next(&mut self) -> Result<&Token, ParseError> {
        if !self.is_at_end()? {
            self.current += 1;
        }
        self.tokens
            .get(self.current - 1)
            .ok_or(ParseError::UnexpectedEof)
    }
}
