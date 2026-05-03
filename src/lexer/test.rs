use crate::lexer::lexer::*;

/// Extrae solo los Token de un fuente, ignorando errores.
fn tokens(src: &str) -> Vec<Token> {
    let (toks, _) = TokenStream::tokenize_all(src);
    toks.into_iter()
        .filter(|t| t.token != Token::Eof)
        .map(|t| t.token)
        .collect()
}

/// Extrae solo los errores léxicos.
fn errors(src: &str) -> Vec<LexError> {
    let (_, errs) = TokenStream::tokenize_all(src);
    errs
}

/// Extrae tokens incluyendo EOF.
fn tokens_with_eof(src: &str) -> Vec<Token> {
    let (toks, _) = TokenStream::tokenize_all(src);
    toks.into_iter().map(|t| t.token).collect()
}

// ── keywords ─────────────────────────────────────────────────────────────

#[test]
fn keywords() {
    let src = "let in if elif else while for function type new \
               inherits is as true false self base protocol extends def";
    assert_eq!(tokens(src), vec![
        Token::Let, Token::In, Token::If, Token::Elif, Token::Else,
        Token::While, Token::For, Token::Function, Token::Type,
        Token::New, Token::Inherits, Token::Is, Token::As,
        Token::True, Token::False, Token::SelfKw, Token::Base,
        Token::Protocol, Token::Extends, Token::Def,
    ]);
}

// ── EOF ───────────────────────────────────────────────────────────────────

#[test]
fn eof_always_present() {
    // fuente vacío → solo EOF
    let toks = tokens_with_eof("");
    assert_eq!(toks, vec![Token::Eof]);
}

#[test]
fn eof_at_end_of_program() {
    let toks = tokens_with_eof("42;");
    assert_eq!(*toks.last().unwrap(), Token::Eof);
}

#[test]
fn eof_position_nonempty() {
    let src = "42";
    let (toks, _) = TokenStream::tokenize_all(src);
    let eof = toks.last().unwrap();
    assert_eq!(eof.token, Token::Eof);
    // EOF debe estar después del último carácter
    assert_eq!(eof.span.start.line, 1);
    assert_eq!(eof.span.start.col, 3); // "42" ocupa cols 1-2, EOF en col 3
}

#[test]
fn eof_position_empty_source() {
    let (toks, _) = TokenStream::tokenize_all("");
    let eof = &toks[0];
    assert_eq!(eof.token, Token::Eof);
    assert_eq!(eof.span.start, Pos { line: 1, col: 1 });
}

#[test]
fn next_token_after_eof_repeats_eof() {
    // next_token() después de EOF debe seguir devolviendo EOF
    let mut stream = TokenStream::new("x");
    let t1 = stream.next_token(); // Ident("x")
    assert_eq!(t1.token, Token::Ident("x".into()));
    let t2 = stream.next_token(); // EOF
    assert_eq!(t2.token, Token::Eof);
    let t3 = stream.next_token(); // EOF otra vez — no panic, no None
    assert_eq!(t3.token, Token::Eof);
}

// ── InternalIdent (_x) ───────────────────────────────────────────────────

#[test]
fn internal_ident_single_underscore_prefix() {
    // _x debe tokenizarse como UN solo token InternalIdent, no _ + x
    assert_eq!(
        tokens("_x"),
        vec![Token::InternalIdent("_x".into())]
    );
}

#[test]
fn internal_ident_compiler_generated() {
    // nombres que el compilador genera en transpilaciones
    assert_eq!(
        tokens("_total"),
        vec![Token::InternalIdent("_total".into())]
    );
    assert_eq!(
        tokens("_IsOddWrapper"),
        vec![Token::InternalIdent("_IsOddWrapper".into())]
    );
}

#[test]
fn internal_ident_only_underscore() {
    // _ solo también es InternalIdent (sin letras después)
    assert_eq!(
        tokens("_"),
        vec![Token::InternalIdent("_".into())]
    );
}

#[test]
fn internal_ident_does_not_split() {
    // _x no se tokeniza como error('_') + Ident("x")
    let errs = errors("_x");
    assert!(errs.is_empty(), "no debe haber errores para _x");

    let toks = tokens("_x");
    assert_eq!(toks.len(), 1, "_x debe ser exactamente un token");
}

#[test]
fn normal_ident_unchanged() {
    // identificador normal no debe verse afectado
    assert_eq!(
        tokens("x_y"),
        vec![Token::Ident("x_y".into())]
    );
}

// ── keywords prefijo de identificador ────────────────────────────────────

#[test]
fn keyword_prefix_of_ident() {
    // "letting" no es Let + Ident("ting"), es Ident("letting")
    assert_eq!(tokens("letting"), vec![Token::Ident("letting".into())]);
    assert_eq!(tokens("forge"),   vec![Token::Ident("forge".into())]);
    assert_eq!(tokens("inform"),  vec![Token::Ident("inform".into())]);
    assert_eq!(tokens("newType"), vec![Token::Ident("newType".into())]);
    assert_eq!(tokens("typeOf"),  vec![Token::Ident("typeOf".into())]);
}

// ── números ───────────────────────────────────────────────────────────────

#[test]
fn number_integer() {
    assert_eq!(tokens("42"), vec![Token::Number("42".into())]);
}

#[test]
fn number_float() {
    assert_eq!(tokens("3.14"), vec![Token::Number("3.14".into())]);
}

#[test]
fn number_incomplete_float() {
    // "3." → Number("3") + Dot  (no crashea, no produce un float inválido)
    assert_eq!(tokens("3."), vec![Token::Number("3".into()), Token::Dot]);
}

#[test]
fn number_member_access() {
    // "x.size()" no consume el punto como parte del número
    let toks = tokens("3.size()");
    assert_eq!(toks[0], Token::Number("3".into()));
    assert_eq!(toks[1], Token::Dot);
    assert_eq!(toks[2], Token::Ident("size".into()));
}

// ── strings ───────────────────────────────────────────────────────────────

#[test]
fn string_simple() {
    assert_eq!(
        tokens(r#""Hello World""#),
        vec![Token::StringLit("Hello World".into())]
    );
}

#[test]
fn string_escape_quote() {
    let toks = tokens(r#""He said \"hi\"""#);
    assert_eq!(toks, vec![Token::StringLit(r#"He said "hi""#.into())]);
}

#[test]
fn string_escape_newline_tab() {
    let toks = tokens(r#""\n\t""#);
    assert_eq!(toks, vec![Token::StringLit("\n\t".into())]);
}

#[test]
fn string_unclosed_gives_error() {
    // string sin cerrar en la misma línea → error léxico
    let errs = errors("\"hola mundo");
    assert!(!errs.is_empty());
}

#[test]
fn string_no_multiline() {
    // salto de línea dentro del string → error, no token válido
    let errs = errors("\"hola\nmundo\"");
    assert!(!errs.is_empty());
}

// ── operadores multi-carácter ─────────────────────────────────────────────

#[test]
fn double_char_ops() {
    let toks = tokens(":= => @@ == != <= >= ->");
    assert_eq!(toks, vec![
        Token::ColonAssign, Token::Arrow, Token::ConcatSpace,
        Token::EqEq, Token::BangEq, Token::LtEq, Token::GtEq,
        Token::ThinArrow,
    ]);
}

#[test]
fn concat_space_maximal_munch() {
    assert_eq!(tokens("@@"),  vec![Token::ConcatSpace]);
    assert_eq!(tokens("@"),   vec![Token::At]);
    assert_eq!(tokens("@ @"), vec![Token::At, Token::At]);
}

#[test]
fn colon_assign_vs_colon_eq() {
    assert_eq!(tokens(":="),  vec![Token::ColonAssign]);
    assert_eq!(tokens(": ="), vec![Token::Colon, Token::Eq]);
}

// ── comentarios ───────────────────────────────────────────────────────────

#[test]
fn comments_skipped() {
    let toks = tokens("42 // comentario\n3.14");
    assert_eq!(toks, vec![Token::Number("42".into()), Token::Number("3.14".into())]);
}

#[test]
fn comment_at_end_of_file() {
    // comentario sin newline al final no debe causar error
    let toks = tokens("42 // sin newline al final");
    assert_eq!(toks, vec![Token::Number("42".into())]);
}

// ── posición (línea/columna) ──────────────────────────────────────────────

#[test]
fn span_line_col() {
    let src = "let\n  x";
    let (toks, _) = TokenStream::tokenize_all(src);
    let let_tok = &toks[0];
    let x_tok   = &toks[1];

    assert_eq!(let_tok.span.start, Pos { line: 1, col: 1 });
    assert_eq!(let_tok.span.end,   Pos { line: 1, col: 3 });
    assert_eq!(x_tok.span.start,   Pos { line: 2, col: 3 });
}

// ── errores sin panic ─────────────────────────────────────────────────────

#[test]
fn error_no_panic() {
    let errs = errors("let x = #42;");
    assert!(!errs.is_empty());
    assert!(errs[0].slice.contains('#'));
}

#[test]
fn error_has_position() {
    let errs = errors("let\n x = #;");
    assert_eq!(errs[0].span.start.line, 2);
}

#[test]
fn errors_dont_stop_tokenization() {
    // '#' es inválido pero los tokens siguientes deben seguir apareciendo
    let (toks, errs) = TokenStream::tokenize_all("let #x = 42;");
    assert!(!errs.is_empty());
    // debe haber tokenizado "let", "x", "=", "42", ";"
    let token_kinds: Vec<_> = toks.iter().map(|t| &t.token).collect();
    assert!(token_kinds.contains(&&Token::Let));
    assert!(token_kinds.contains(&&Token::Ident("x".into())));
}

// ── programas completos ───────────────────────────────────────────────────

#[test]
fn hello_world() {
    let toks = tokens(r#"print("Hello World");"#);
    assert_eq!(toks, vec![
        Token::Ident("print".into()),
        Token::LParen,
        Token::StringLit("Hello World".into()),
        Token::RParen,
        Token::Semicolon,
    ]);
}

#[test]
fn type_declaration() {
    let toks = tokens("type Point(x: Number, y: Number) { x = x; }");
    assert_eq!(toks[0], Token::Type);
    assert_eq!(toks[1], Token::Ident("Point".into()));
    assert_eq!(toks[2], Token::LParen);
}

#[test]
fn vector_implicit() {
    let toks = tokens("[x^2 | x in range(1,10)]");
    assert_eq!(toks[0], Token::LBracket);
    assert!(toks.contains(&Token::Pipe));
    assert!(toks.contains(&Token::In));
}

#[test]
fn functor_type() {
    let toks = tokens("(Number) -> Boolean");
    assert!(toks.contains(&Token::ThinArrow));
}

#[test]
fn destructive_assign() {
    let toks = tokens("a := 1;");
    assert_eq!(toks, vec![
        Token::Ident("a".into()),
        Token::ColonAssign,
        Token::Number("1".into()),
        Token::Semicolon,
    ]);
}

#[test]
fn transpiled_code_with_internal_idents() {
    // código generado por el compilador usa _ident
    let toks = tokens("let _total = 0 in _total := _total + 1;");
    assert_eq!(toks[1], Token::InternalIdent("_total".into()));
    assert_eq!(toks[5], Token::InternalIdent("_total".into()));
}
