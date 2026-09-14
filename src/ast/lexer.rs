use super::token::{Token, TokenType};
use std::borrow::Cow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unknown lexeme at line {line}, got {got}")]
    UnknownLexeme { line: usize, got: u8 },
    #[error("Broken CRLF at line {line}")]
    Crlf { line: usize },
    #[error("Expected ':' after BEGIN/END at line {line}")]
    ExpectedColon { line: usize },
}

#[derive(Debug, Default)]
pub struct Lexer {
    source: Vec<u8>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Lexer {
    /// creates a new [Lexer] out of a source
    ///
    /// `src` is unfolded (RFC 5545 §3.1: CRLF followed by a single SPACE/HTAB
    /// is a soft line break, not a real one) before scanning ever sees it, so
    /// every downstream token position is a *logical* (post-unfolding)
    /// content line rather than a raw physical one.
    pub fn new(src: &[u8]) -> Self {
        Self {
            source: unfold::unfold(src),
            ..Default::default()
        }
    }

    /// scans the source for tokens
    pub fn scan(mut self) -> Result<Vec<Token>, LexerError> {
        while !self.is_at_end() {
            self.start = self.current;
            let c = self.next();

            match c {
                b'\r' => {
                    if self.match_next(b'\n') {
                        self.add_token(TokenType::Crlf, None);
                        self.line += 1;
                    } else {
                        return Err(LexerError::Crlf { line: self.line });
                    }
                }
                b'\n' => {
                    self.add_token(TokenType::Crlf, None);
                    self.line += 1;
                }
                b' ' | b'\t' => {}
                c if c.is_ascii_alphanumeric() => self.line_content()?,
                _ => {
                    return Err(LexerError::UnknownLexeme {
                        line: self.line,
                        got: c,
                    });
                }
            }
        }

        self.tokens
            .push(Token::new(TokenType::Eof, b"", None, self.line));

        Ok(self.tokens)
    }

    /// Scans one content line's name field (`BEGIN`/`END`/a property name),
    /// then dispatches to either a component pair ([`Self::component`]) or
    /// a generic [`TokenType::Property`] token holding the unparsed
    /// remainder of the line ([`Self::property`]).
    fn line_content(&mut self) -> Result<(), LexerError> {
        self.name_chars();
        // Owned rather than the `Cow` `fold_upper` returns: it borrows
        // `self.source`, and the match arms below need `&mut self`, which a
        // live borrow of one of `self`'s fields would block.
        let name = Self::fold_upper(&self.source[self.start..self.current])
            .into_owned();

        match name.as_slice() {
            b"BEGIN" => self.component(TokenType::Begin),
            b"END" => self.component(TokenType::End),
            _ => {
                self.property(&name);
                Ok(())
            }
        }
    }

    /// `BEGIN`/`END` are followed by `:` and a component name — the only
    /// structure the parser actually needs from the lexer, since it drives
    /// recursion into (or out of) a component builder.
    fn component(&mut self, tt: TokenType) -> Result<(), LexerError> {
        if self.is_at_end() || self.next() != b':' {
            return Err(LexerError::ExpectedColon { line: self.line });
        }

        let comp_start = self.current;
        self.name_chars();
        let comp_name =
            Self::fold_upper(&self.source[comp_start..self.current]);

        let lex: &[u8] = if matches!(tt, TokenType::Begin) {
            b"BEGIN"
        } else {
            b"END"
        };
        self.tokens.push(Token::new(
            tt,
            lex,
            Some(comp_name.as_ref()),
            self.line,
        ));
        Ok(())
    }

    /// Any content line that isn't `BEGIN`/`END`: capture the raw,
    /// unparsed remainder from right after the name to end of line —
    /// `*(";" param) ":" value` — and hand it off whole. Splitting params
    /// from the value (honoring DQUOTE-ing) is the matching property
    /// type's job, not the lexer's; see `crate::properties::value_start`.
    fn property(&mut self, name: &[u8]) {
        let rest_start = self.current;
        let rest = &self.source[self.current..];
        self.current +=
            memchr::memchr2(b'\r', b'\n', rest).unwrap_or(rest.len());
        let remainder = &self.source[rest_start..self.current];

        self.tokens.push(Token::new(
            TokenType::Property,
            name,
            Some(remainder),
            self.line,
        ));
    }

    /// advances past a run of name characters (`ALPHA` / `DIGIT` / `-`),
    /// shared by property names and component names alike
    fn name_chars(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == b'-' {
            self.next();
        }
    }

    /// Real-world input follows the RFC 5545 §2 convention of writing
    /// names in uppercase already, so only pay for the fold (and its
    /// allocation) when the text actually contains a lowercase byte.
    fn fold_upper(bytes: &[u8]) -> Cow<'_, [u8]> {
        if bytes.iter().any(u8::is_ascii_lowercase) {
            Cow::Owned(bytes.to_ascii_uppercase())
        } else {
            Cow::Borrowed(bytes)
        }
    }

    /// adds a new token to self
    fn add_token(&mut self, tt: TokenType, lit: Option<&[u8]>) {
        let lex = &self.source[self.start..self.current];
        self.tokens.push(Token::new(tt, lex, lit, self.line));
    }

    /// advances the lexer, returning the consumed byte
    fn next(&mut self) -> u8 {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    /// looks at what the next byte is
    fn peek(&self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.source[self.current]
        }
    }

    fn match_next(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            false
        } else if self.source[self.current] == expected {
            self.current += 1;
            true
        } else {
            false
        }
    }

    /// checks that the lexer has read all of the source
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

pub(crate) mod unfold;
pub use unfold::unfold;

#[cfg(test)]
mod tests;
