#![allow(dead_code)]

use logos::{Logos, SpannedIter};

// ─────────────────────────────────────────────────────────────────────────────
// Posición en el código fuente (línea/columna base-1)
// ─────────────────────────────────────────────────────────────────────────────

/// Posición en el código fuente. Ambos valores son base-1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// Span de un token: posición inicial y final en el fuente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tabla de líneas — convierte byte-offset → (línea, columna)
// ─────────────────────────────────────────────────────────────────────────────

/// Índice de offsets de inicio de cada línea.
/// Permite convertir byte-offset → (línea, columna) en O(log n).
pub struct LineIndex {
    /// starts[i] = byte-offset donde comienza la línea i+1 (base-1).
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut starts = vec![0usize];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
            }
        }
        Self { starts }
    }

    /// Convierte un byte-offset a Pos (línea y columna base-1).
    pub fn pos(&self, offset: usize) -> Pos {
        let line_idx = self.starts.partition_point(|&s| s <= offset) - 1;
        let col = offset - self.starts[line_idx] + 1;
        Pos {
            line: line_idx + 1,
            col,
        }
    }

    /// Convierte un rango de bytes en un Span.
    pub fn span(&self, range: std::ops::Range<usize>) -> Span {
        Span {
            start: self.pos(range.start),
            // end apunta al último byte del token, no al siguiente
            end: self.pos(range.end.saturating_sub(1)),
        }
    }

    /// Posición del EOF: un carácter después del último.
    pub fn eof_pos(&self, source_len: usize) -> Pos {
        if source_len == 0 {
            Pos { line: 1, col: 1 }
        } else {
            let last = self.pos(source_len - 1);
            Pos {
                line: last.line,
                col: last.col + 1,
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Token — definición completa del lenguaje HULK
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]  // whitespace (no significativo en HULK)
#[logos(skip r"//[^\n]*")]    // comentarios de línea
pub enum Token {

    // ── Palabras clave ────────────────────────────────────────────────────────
    #[token("let")]      Let,
    #[token("in")]       In,
    #[token("if")]       If,
    #[token("elif")]     Elif,
    #[token("else")]     Else,
    #[token("while")]    While,
    #[token("for")]      For,
    #[token("function")] Function,
    #[token("type")]     Type,
    #[token("new")]      New,
    #[token("inherits")] Inherits,
    #[token("is")]       Is,
    #[token("as")]       As,
    #[token("true")]     True,
    #[token("false")]    False,
    #[token("self")]     SelfKw,
    #[token("base")]     Base,
    #[token("protocol")] Protocol,
    #[token("extends")]  Extends,
    #[token("def")]      Def,       // macros (sección A.14)

    // ── Builtins matemáticos ───────────────────────────────────────────────

    #[token("sqrt")]     Sqrt,
    #[token("sin")]      Sin,
    #[token("cos")]      Cos,
    #[token("exp")]      Exp,
    #[token("log")]      Log,
    #[token("rand")]     Rand,
    #[token("PI", priority = 3)] Pi,
    #[token("E", priority = 3)]  E,

    // ── Literales ─────────────────────────────────────────────────────────────

    /// Número: entero o flotante.
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().to_owned())]
    Number(String),

    /// String literal con soporte de escapes: \" \n \t \\
    #[regex(r#""([^\n"\\]|\\.)*""#, lex_string)]
    StringLit(String),

    // ── Identificadores ───────────────────────────────────────────────────────

    /// Identificador válido en HULK: comienza con letra, sigue con letras,
    /// dígitos o guión bajo.
    /// Ejemplos: x  x0  camelCase  TitleCase  snake_case
    #[regex(r"[a-zA-Z][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Ident(String),

    /// Identificador que comienza con '_'.
    ///
    /// En código de usuario es un error semántico (no léxico).
    /// El compilador los genera internamente en transpilaciones
    /// (_total, _IsOddWrapper, etc.) por lo que deben ser tokens válidos
    /// para que el mismo lexer pueda re-tokenizar código transpilado.
    ///
    /// El parser rechaza `InternalIdent` en posiciones de código de usuario
    /// y los acepta solo en código generado por el compilador.
    #[regex(r"_[a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    InternalIdent(String),

    // ── Operadores de dos o más caracteres (ANTES de los de un carácter) ─────

    #[token(":=")] ColonAssign,   // asignación destructiva
    #[token("=>")] Arrow,         // cuerpo de función/lambda inline
    #[token("@@")] ConcatSpace,   // concatenación con espacio (≡ @ " " @)
    #[token("==")] EqEq,          // igualdad
    #[token("!=")] BangEq,        // desigualdad
    #[token("<=")] LtEq,          // menor o igual
    #[token(">=")] GtEq,          // mayor o igual
    #[token("->")] ThinArrow,     // tipo functor: (Number) -> Boolean

    // ── Operadores de un carácter ─────────────────────────────────────────────

    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Star,
    #[token("/")] Slash,
    #[token("%")] Percent,
    #[token("^")] Caret,       // potencia
    #[token("@")] At,          // concatenación simple
    #[token("&")] Amp,         // AND booleano
    #[token("|")] Pipe,        // OR booleano / separador en vector implícito
    #[token("!")] Bang,        // NOT booleano
    #[token("<")] Lt,
    #[token(">")] Gt,
    #[token("=")] Eq,          // asignación en let/atributos

    // ── Puntuación ────────────────────────────────────────────────────────────

    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token(";")] Semicolon,
    #[token(",")] Comma,
    #[token(".")] Dot,
    #[token(":")] Colon,       // anotación de tipo: x: Number
    #[token("$")] Dollar,      // placeholder en macros: $iter

    // ── Centinela de fin de archivo ───────────────────────────────────────────
    //
    // No lleva #[token] ni #[regex]: logos nunca lo emite.
    // Lo inserta el wrapper `TokenStream` al agotar el iterador interno.
    // El parser usa este token para detectar EOF sin tener que manejar
    // Option en cada llamada a peek()/advance().
    Eof,
}

// ─── Callbacks de logos ───────────────────────────────────────────────────────


/// Expande las secuencias de escape de un string literal.
/// logos ya garantizó que el patrón casa con `"([^"\\]|\\.)*"`.
fn lex_string(lex: &mut logos::Lexer<Token>) -> Option<String> {
    let raw = lex.slice();
    let inner = &raw[1..raw.len() - 1]; // quitar comillas externas
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next()? {
                '"'  => out.push('"'),
                'n'  => out.push('\n'),
                't'  => out.push('\t'),
                '\\' => out.push('\\'),
                // escape desconocido: preservar literalmente
                other => { out.push('\\'); out.push(other); }
            }
        } else {
            out.push(c);
        }
    }
    Some(out)
}

// ─────────────────────────────────────────────────────────────────────────────
// Error léxico
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub msg: String,
    pub span: Span,
    /// El texto que causó el error.
    pub slice: String,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[LexError {}] {} (encontrado {:?})", self.span, self.msg, self.slice)
    }
}

impl std::error::Error for LexError {}

// ─────────────────────────────────────────────────────────────────────────────
// Token con posición
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
    /// Texto original en el fuente. Vacío para Token::Eof.
    pub slice: String,
}

pub type LexResult = Result<SpannedToken, LexError>;

// ─────────────────────────────────────────────────────────────────────────────
// Lexer interno — wrappea logos, convierte offsets a línea/columna
// ─────────────────────────────────────────────────────────────────────────────

struct InnerLexer<'src> {
    inner: SpannedIter<'src, Token>,
    index: LineIndex,
    source: &'src str,
}

impl<'src> InnerLexer<'src> {
    fn new(source: &'src str) -> Self {
        Self {
            inner: Token::lexer(source).spanned(),
            index: LineIndex::new(source),
            source,
        }
    }
}

impl<'src> Iterator for InnerLexer<'src> {
    type Item = LexResult;

    fn next(&mut self) -> Option<Self::Item> {
        let (result, byte_range) = self.inner.next()?;
        let span  = self.index.span(byte_range.clone());
        let slice = self.source[byte_range].to_owned();

        match result {
            Ok(token) => Some(Ok(SpannedToken { token, span, slice })),
            Err(())   => Some(Err(LexError {
                msg: "carácter inesperado".into(),
                span,
                slice,
            })),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TokenStream — interfaz pública
//
// Es el único tipo que el parser debe usar. Garantiza:
//   1. El último token siempre es Token::Eof (nunca devuelve None).
//   2. Los errores léxicos se acumulan en `errors` y no interrumpen el flujo,
//      permitiendo que el parser continúe y reporte todos los errores juntos.
// ─────────────────────────────────────────────────────────────────────────────

pub struct TokenStream<'src> {
    inner:   InnerLexer<'src>,
    index:   LineIndex,          // necesario para calcular la posición del EOF
    src_len: usize,
    /// Errores léxicos encontrados durante la tokenización.
    /// El parser puede consultarlos después de parsear.
    pub errors: Vec<LexError>,
}

impl<'src> TokenStream<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            inner:       InnerLexer::new(source),
            index:       LineIndex::new(source),
            src_len:     source.len(),
            errors:      Vec::new(),
        }
    }

    /// Tokeniza todo el fuente de una vez.
    /// Devuelve los tokens (incluyendo EOF al final) y acumula los errores.
    pub fn tokenize_all(source: &'src str) -> (Vec<SpannedToken>, Vec<LexError>) {
        let mut stream = Self::new(source);
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        loop {
            let tok = stream.next_token();
            let is_eof = tok.token == Token::Eof;
            tokens.push(tok);
            if is_eof { break; }
        }
        errors.extend(stream.errors.drain(..));
        (tokens, errors)
    }

    /// Avanza y devuelve el siguiente token.
    /// Nunca devuelve un error: los errores se acumulan en `self.errors`
    /// y se salta el carácter problemático continuando con el siguiente token.
    /// Garantiza que siempre termina en Token::Eof.
    pub fn next_token(&mut self) -> SpannedToken {
        loop {
            match self.inner.next() {
                Some(Ok(tok)) => return tok,

                Some(Err(err)) => {
                    // acumular el error y continuar — nunca panic
                    self.errors.push(err);
                    // seguir iterando hasta encontrar un token válido o EOF
                }

                None => {
                    // fuente agotado — emitir EOF una sola vez y luego
                    // repetirlo indefinidamente para que el parser pueda
                    // hacer peek() sin preocuparse por Option
                    let eof_pos = self.index.eof_pos(self.src_len);
                    return SpannedToken {
                        token: Token::Eof,
                        span:  Span { start: eof_pos, end: eof_pos },
                        slice: String::new(),
                    };
                }
            }
        }
    }

    /// ¿Hay errores léxicos acumulados?
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}