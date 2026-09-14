use super::*;

/// `Token` has no accessors and derives only `Debug`, so equality here
/// is checked by comparing the derived Debug output against tokens
/// built with the same public [`Token::new`] constructor the lexer uses.
fn assert_tokens(actual: Vec<Token>, expected: Vec<Token>) {
    assert_eq!(format!("{:?}", actual), format!("{:?}", expected));
}

fn lex(src: &[u8]) -> Result<Vec<Token>, LexerError> {
    Lexer::new(src).scan()
}

#[test]
fn begin_end_carry_the_component_name_as_literal() {
    let tokens = lex(b"BEGIN:VEVENT\r\nEND:VEVENT\r\n").unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(TokenType::Begin, b"BEGIN", Some(b"VEVENT"), 0),
            Token::new(TokenType::Crlf, b"\r\n", None, 0),
            Token::new(TokenType::End, b"END", Some(b"VEVENT"), 1),
            Token::new(TokenType::Crlf, b"\r\n", None, 1),
            Token::new(TokenType::Eof, b"", None, 2),
        ],
    );
}

#[test]
fn begin_end_component_name_is_case_folded() {
    let tokens = lex(b"begin:vevent\r\n").unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(TokenType::Begin, b"BEGIN", Some(b"VEVENT"), 0),
            Token::new(TokenType::Crlf, b"\r\n", None, 0),
            Token::new(TokenType::Eof, b"", None, 1),
        ],
    );
}

#[test]
fn begin_without_colon_errors() {
    let res = lex(b"BEGIN VEVENT\r\n");
    assert!(matches!(res, Err(LexerError::ExpectedColon { line: 0 })));
}

#[test]
fn a_property_line_becomes_one_property_token() {
    // The lexer doesn't split params from the value, or interpret DQUOTEs
    // at all — the whole `*(";" param) ":" value` remainder is opaque to
    // it. That's `crate::properties::value_start`'s job, working directly
    // off these raw bytes.
    let tokens =
        lex(b"RECURRENCE-ID;RANGE=THISANDFUTURE:20240402T100000\r\n").unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(
                TokenType::Property,
                b"RECURRENCE-ID",
                Some(b";RANGE=THISANDFUTURE:20240402T100000"),
                0,
            ),
            Token::new(TokenType::Crlf, b"\r\n", None, 0),
            Token::new(TokenType::Eof, b"", None, 1),
        ],
    );
}

#[test]
fn property_name_is_case_folded_but_remainder_is_left_verbatim() {
    // Names are case-insensitive per §2; the value/param text after the
    // name is not touched or folded by the lexer at all.
    let tokens = lex(b"rrule:FREQ=Daily\r\n").unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(TokenType::Property, b"RRULE", Some(b":FREQ=Daily"), 0),
            Token::new(TokenType::Crlf, b"\r\n", None, 0),
            Token::new(TokenType::Eof, b"", None, 1),
        ],
    );
}

#[test]
fn unrecognized_property_name_is_still_a_property_token() {
    // The lexer has no concept of a "known" vs "unknown" property name —
    // that distinction (falling back to Xprop/Iana) lives entirely in the
    // name -> parser dispatch table, not here.
    let tokens = lex(b"X-CUSTOM-PROP:value\r\n").unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(
                TokenType::Property,
                b"X-CUSTOM-PROP",
                Some(b":value"),
                0,
            ),
            Token::new(TokenType::Crlf, b"\r\n", None, 0),
            Token::new(TokenType::Eof, b"", None, 1),
        ],
    );
}

#[test]
fn property_remainder_may_contain_arbitrary_punctuation_unparsed() {
    // Semicolons, colons, commas and quotes inside the remainder all pass
    // through untouched raw bytes — the lexer performs no quote-aware
    // splitting of any kind.
    let tokens =
        lex(b"ATTENDEE;DELEGATED-FROM=\"a,b\":mailto:foo@example.com\r\n")
            .unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(
                TokenType::Property,
                b"ATTENDEE",
                Some(b";DELEGATED-FROM=\"a,b\":mailto:foo@example.com"),
                0,
            ),
            Token::new(TokenType::Crlf, b"\r\n", None, 0),
            Token::new(TokenType::Eof, b"", None, 1),
        ],
    );
}

#[test]
fn line_advances_after_each_crlf() {
    // §3.1: a logical content line ends at CRLF. Every token scanned
    // after a CRLF belongs to the next (post-unfolding) line, so `line`
    // must increment once per CRLF consumed.
    let tokens = lex(b"UID:foo\r\nDTSTAMP:20240102T090000Z\r\n").unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(TokenType::Property, b"UID", Some(b":foo"), 0),
            Token::new(TokenType::Crlf, b"\r\n", None, 0),
            Token::new(
                TokenType::Property,
                b"DTSTAMP",
                Some(b":20240102T090000Z"),
                1,
            ),
            Token::new(TokenType::Crlf, b"\r\n", None, 1),
            Token::new(TokenType::Eof, b"", None, 2),
        ],
    );
}

#[test]
fn lone_cr_followed_by_non_lf_errors_gracefully() {
    // A property's raw-remainder scan (`property()`) stops at `\r`
    // without consuming it, so this exercises the main loop's CRLF
    // handling specifically, not `component()`'s `BEGIN`/`END` colon
    // check (see `begin_without_colon_errors` for that path).
    let res = lex(b"UID:foo\rX");
    assert!(matches!(res, Err(LexerError::Crlf { line: 0 })));
}

#[test]
fn trailing_bare_cr_errors_gracefully_instead_of_panicking() {
    // A `\r` as the very last byte of the source, with no trailing `\n`,
    // is malformed input, not a memory-safety incident: the lexer must
    // return `LexerError::Crlf`, never read past the end of the buffer.
    let res = lex(b"UID:foo\r");
    assert!(matches!(res, Err(LexerError::Crlf { .. })));
}

#[test]
fn folded_content_line_is_unfolded_before_scanning() {
    // §3.1's own worked example: a folded content line must scan as the
    // single logical property it represents, not three separate lines.
    let tokens = lex(
        b"DESCRIPTION:This is a lo\r\n ng description\r\n  that exists on a long line.\r\n",
    )
    .unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(
                TokenType::Property,
                b"DESCRIPTION",
                Some(
                    b":This is a long description that exists on a long line.",
                ),
                0,
            ),
            Token::new(TokenType::Crlf, b"\r\n", None, 0),
            Token::new(TokenType::Eof, b"", None, 1),
        ],
    );
}

#[test]
fn fold_inside_a_begin_end_component_name_is_unfolded() {
    // Folding can split any two characters, including mid-keyword — the
    // lexer must see `VEVENT`, not `VE` followed by a bogus continuation.
    let tokens = lex(b"BEGIN:VE\r\n VENT\r\n").unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(TokenType::Begin, b"BEGIN", Some(b"VEVENT"), 0),
            Token::new(TokenType::Crlf, b"\r\n", None, 0),
            Token::new(TokenType::Eof, b"", None, 1),
        ],
    );
}

#[test]
fn unknown_lexeme_at_start_of_line_errors() {
    let res = lex(b"!oops\r\n");
    assert!(matches!(
        res,
        Err(LexerError::UnknownLexeme { line: 0, got: b'!' })
    ));
}

#[test]
fn bare_lf_is_accepted_as_a_line_terminator() {
    // RFC 5545 §3.1 specifies CRLF, but real-world LF-only `.ics` files
    // (produced without CRLF normalization) are common and every mainstream
    // parser accepts them. A bare `\n` must terminate a content line — not
    // be silently swallowed as whitespace, which used to concatenate it
    // into the next line's content.
    let tokens = lex(b"UID:foo\nDTSTAMP:20240102T090000Z\n").unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(TokenType::Property, b"UID", Some(b":foo"), 0),
            Token::new(TokenType::Crlf, b"\n", None, 0),
            Token::new(
                TokenType::Property,
                b"DTSTAMP",
                Some(b":20240102T090000Z"),
                1,
            ),
            Token::new(TokenType::Crlf, b"\n", None, 1),
            Token::new(TokenType::Eof, b"", None, 2),
        ],
    );
}

#[test]
fn bare_lf_terminated_begin_end_still_carries_component_name() {
    let tokens = lex(b"BEGIN:VEVENT\nEND:VEVENT\n").unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(TokenType::Begin, b"BEGIN", Some(b"VEVENT"), 0),
            Token::new(TokenType::Crlf, b"\n", None, 0),
            Token::new(TokenType::End, b"END", Some(b"VEVENT"), 1),
            Token::new(TokenType::Crlf, b"\n", None, 1),
            Token::new(TokenType::Eof, b"", None, 2),
        ],
    );
}

#[test]
fn lone_cr_not_followed_by_lf_still_errors_when_file_is_otherwise_lf_only() {
    // Mixing in a bare `\r` (not part of a `\r\n` pair) must still be
    // rejected even in an otherwise LF-terminated file — accepting bare
    // `\n` doesn't mean accepting bare `\r` too.
    let res = lex(b"UID:foo\rX\n");
    assert!(matches!(res, Err(LexerError::Crlf { line: 0 })));
}

#[test]
fn folded_content_line_with_bare_lf_is_unfolded_before_scanning() {
    // Mirrors `folded_content_line_is_unfolded_before_scanning`, but for a
    // file whose fold sequences use bare `\n` + WSP instead of `\r\n` + WSP
    // (see `unfold::unfold`'s doc comment).
    let tokens = lex(
        b"DESCRIPTION:This is a lo\n ng description\n  that exists on a long line.\n",
    )
    .unwrap();
    assert_tokens(
        tokens,
        vec![
            Token::new(
                TokenType::Property,
                b"DESCRIPTION",
                Some(
                    b":This is a long description that exists on a long line.",
                ),
                0,
            ),
            Token::new(TokenType::Crlf, b"\n", None, 0),
            Token::new(TokenType::Eof, b"", None, 1),
        ],
    );
}
