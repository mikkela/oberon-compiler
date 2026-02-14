use std::fs;
use std::path::Path;
use oberon_compiler::lexer::{Lexer, Token, TokenKind};

fn kinds(input: &str) -> Vec<TokenKind> {
    let mut lx = Lexer::new(input);
    let mut out = Vec::new();

    loop {
        let tok = lx.next_token().expect("lexing should succeed");
        let kind = tok.kind.clone();
        out.push(kind.clone());
        if matches!(kind, TokenKind::Eof) {
            break;
        }
    }

    out
}

fn lex_all(input: &str) -> Vec<Token> {
    let mut lx = Lexer::new(input);
    let mut out = Vec::new();
    loop {
        let tok = lx.next_token().expect("lexing should succeed");
        let is_eof = matches!(tok.kind, TokenKind::Eof);
        out.push(tok);
        if is_eof {
            break;
        }
    }
    out
}

fn render_tokens(input: &str) -> String {
    let toks = lex_all(input);
    let mut s = String::new();

    for t in toks {
        use std::fmt::Write;
        match &t.kind {
            TokenKind::Number(n) =>
                writeln!(&mut s, "Number({n}) @ [{}..{}]", t.span.start, t.span.end).unwrap(),
            TokenKind::Identifier(id) =>
                writeln!(&mut s, "Identifier({id}) @ [{}..{}]", t.span.start, t.span.end).unwrap(),
            TokenKind::KeywordIf =>
                writeln!(&mut s, "Keyword(IF) @ [{}..{}]", t.span.start, t.span.end).unwrap(),
            TokenKind::Less =>
                writeln!(&mut s, "Symbol(<) @ [{}..{}]", t.span.start, t.span.end).unwrap(),
            TokenKind::LessEqual =>
                writeln!(&mut s, "Symbol(<=) @ [{}..{}]", t.span.start, t.span.end).unwrap(),
            TokenKind::Eof =>
                writeln!(&mut s, "EOF @ [{}..{}]", t.span.start, t.span.end).unwrap(),
        }
    }

    s
}

#[test]
fn lexes_number() {
    assert_eq!(kinds("123"), vec![TokenKind::Number(123), TokenKind::Eof]);
}

#[test]
fn lexes_identifier() {
    assert_eq!(
        kinds("hello"),
        vec![TokenKind::Identifier("hello".to_string()), TokenKind::Eof]
    );
}

#[test]
fn lexes_keyword_if_case_insensitive() {
    assert_eq!(kinds("IF"), vec![TokenKind::KeywordIf, TokenKind::Eof]);
    assert_eq!(kinds("if"), vec![TokenKind::KeywordIf, TokenKind::Eof]);
    assert_eq!(kinds("If"), vec![TokenKind::KeywordIf, TokenKind::Eof]);
}

#[test]
fn lexes_less_and_less_equal() {
    assert_eq!(kinds("<"), vec![TokenKind::Less, TokenKind::Eof]);
    assert_eq!(kinds("<="), vec![TokenKind::LessEqual, TokenKind::Eof]);
}

#[test]
fn skips_whitespace_between_tokens() {
    assert_eq!(
        kinds("IF   a  <=   10"),
        vec![
            TokenKind::KeywordIf,
            TokenKind::Identifier("a".to_string()),
            TokenKind::LessEqual,
            TokenKind::Number(10),
            TokenKind::Eof
        ]
    );
}

#[test]
fn identifier_does_not_greedy_consume_symbol() {
    assert_eq!(
        kinds("a<10"),
        vec![
            TokenKind::Identifier("a".to_string()),
            TokenKind::Less,
            TokenKind::Number(10),
            TokenKind::Eof
        ]
    );
}

#[test]
fn spans_are_byte_offsets() {
    let mut lx = Lexer::new("IF a<=10");
    let t1 = lx.next_token().unwrap(); // IF
    let t2 = lx.next_token().unwrap(); // a
    let t3 = lx.next_token().unwrap(); // <=
    let t4 = lx.next_token().unwrap(); // 10
    let t5 = lx.next_token().unwrap(); // EOF

    assert_eq!(t1.span.start, 0);
    assert_eq!(t1.span.end, 2);

    assert_eq!(t2.span.start, 3);
    assert_eq!(t2.span.end, 4);

    assert_eq!(t3.span.start, 4);
    assert_eq!(t3.span.end, 6);

    assert_eq!(t4.span.start, 6);
    assert_eq!(t4.span.end, 8);

    assert_eq!(t5.span.start, 8);
    assert_eq!(t5.span.end, 8);
}

#[test]
fn unexpected_character_is_an_error_with_span() {
    let mut lx = Lexer::new("@");
    let err = lx.next_token().unwrap_err();

    assert!(err.message.contains("Unexpected character"));
    assert_eq!(err.span.start, 0);
    assert_eq!(err.span.end, 1);
}

#[test]
fn number_followed_by_identifier_splits() {
    // "12a" should lex as Number(12) then Identifier("a") with current rules
    assert_eq!(
        kinds("12a"),
        vec![
            TokenKind::Number(12),
            TokenKind::Identifier("a".to_string()),
            TokenKind::Eof
        ]
    );
}

#[test]
fn prop_tokens_progress_monotonically() {
    let toks = lex_all("IF\tA<=10\r\nIF B<2");
    for w in toks.windows(2) {
        let a = &w[0];
        let b = &w[1];
        assert!(
            a.span.end <= b.span.start,
            "tokens overlap or go backwards: {:?} then {:?}",
            a, b
        );
        assert!(
            a.span.start <= a.span.end,
            "invalid span: {:?}",
            a
        );
    }
}

#[test]
fn prop_no_empty_spans_except_eof() {
    let toks = lex_all("IF a<=10");
    for t in toks {
        match t.kind {
            TokenKind::Eof => assert_eq!(t.span.start, t.span.end),
            _ => assert!(
                t.span.start < t.span.end,
                "non-EOF token should have non-empty span: {:?}",
                t
            ),
        }
    }
}

#[test]
fn prop_eof_is_last_and_unique() {
    let toks = lex_all("IF a<=10");
    let eof_count = toks.iter().filter(|t| matches!(t.kind, TokenKind::Eof)).count();
    assert_eq!(eof_count, 1, "expected exactly one EOF token");
    assert!(matches!(toks.last().unwrap().kind, TokenKind::Eof), "EOF must be last");
}

#[test]
fn spans_with_tabs_are_byte_offsets() {
    // "IF\tX"
    // bytes: I(0) F(1) \t(2) X(3)
    let toks = lex_all("IF\tX");
    assert_eq!(toks[0].kind, TokenKind::KeywordIf);
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 2);

    assert_eq!(toks[1].kind, TokenKind::Identifier("X".to_string()));
    assert_eq!(toks[1].span.start, 3);
    assert_eq!(toks[1].span.end, 4);

    assert!(matches!(toks[2].kind, TokenKind::Eof));
    assert_eq!(toks[2].span.start, 4);
    assert_eq!(toks[2].span.end, 4);
}

#[test]
fn spans_with_crlf_are_byte_offsets() {
    // "IF\r\nA"
    // bytes: I(0) F(1) \r(2) \n(3) A(4)
    let toks = lex_all("IF\r\nA");
    assert_eq!(toks[0].kind, TokenKind::KeywordIf);
    assert_eq!(toks[0].span.start, 0);
    assert_eq!(toks[0].span.end, 2);

    assert_eq!(toks[1].kind, TokenKind::Identifier("A".to_string()));
    assert_eq!(toks[1].span.start, 4);
    assert_eq!(toks[1].span.end, 5);

    assert!(matches!(toks[2].kind, TokenKind::Eof));
    assert_eq!(toks[2].span.start, 5);
    assert_eq!(toks[2].span.end, 5);
}


#[test]
fn golden_token_stream_example() {
    let input = "IF a<=10\r\nIF b<2";
    let got = render_tokens(input);

    // This string is your “golden” expectation.
    // If lexer behavior changes, this test will show exactly what changed.
    let expected = "\
Keyword(IF) @ [0..2]
Identifier(a) @ [3..4]
Symbol(<=) @ [4..6]
Number(10) @ [6..8]
Keyword(IF) @ [10..12]
Identifier(b) @ [13..14]
Symbol(<) @ [14..15]
Number(2) @ [15..16]
EOF @ [16..16]
";

    assert_eq!(got, expected);
}

#[test]
fn golden_lexer_output() {
    let input = "IF a<=10\r\nIF b<2";
    let actual = render_tokens(input);

    let golden_path = Path::new("tests/golden/lexer_example.txt");
    let expected = fs::read_to_string(golden_path)
        .expect("golden file must exist");

    assert_eq!(actual, expected);
}