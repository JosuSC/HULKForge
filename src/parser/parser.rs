use crate::lexer::{Token, SpannedToken, TokenStream, Span};

// ─────────────────────────────────────────────────────────────────────────────
// Parser
// ─────────────────────────────────────────────────────────────────────────────

pub struct Parser<'src> {
    tokens: TokenStream<'src>,
    current: SpannedToken,
    previous: Option<SpannedToken>,

    pub errors: Vec<String>,
}

impl<'src> Parser<'src> {

    // ─────────────────────────────────────────────────────────────────────────
    // Inicialización
    // ─────────────────────────────────────────────────────────────────────────

    pub fn new(mut tokens: TokenStream<'src>) -> Self {
        let first = tokens.next_token();

        Self {
            tokens,
            current: first,
            previous: None,
            errors: Vec::new(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Core navigation
    // ─────────────────────────────────────────────────────────────────────────

    /// Avanza al siguiente token
    pub fn advance(&mut self) {
        self.previous = Some(self.current.clone());
        self.current = self.tokens.next_token();
    }

    /// Token actual (sin consumir)
    pub fn peek(&self) -> &Token {
        &self.current.token
    }

    /// Token anterior
    pub fn previous(&self) -> Option<&SpannedToken> {
        self.previous.as_ref()
    }

    /// ¿Estamos en EOF?
    pub fn is_at_end(&self) -> bool {
        self.current.token == Token::Eof
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Matching básico
    // ─────────────────────────────────────────────────────────────────────────

    /// Verifica si el token actual es igual (exact match)
    pub fn check(&self, token: &Token) -> bool {
        &self.current.token == token
    }

    /// Consume si coincide exactamente
    pub fn matches(&mut self, expected: &Token) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume uno de varios tokens posibles
    pub fn match_any(&mut self, tokens: &[Token]) -> bool {
        for t in tokens {
            if &self.current.token == t {
                self.advance();
                return true;
            }
        }
        false
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Matching por "tipo" (clave para tokens con datos)
    // ─────────────────────────────────────────────────────────────────────────

    /// Verifica por patrón 
    pub fn check_kind(&self, f: fn(&Token) -> bool) -> bool {
        f(&self.current.token)
    }

    /// Consume si cumple el patrón
    pub fn match_kind(&mut self, f: fn(&Token) -> bool) -> bool {
        if f(&self.current.token) {
            self.advance();
            true
        } else {
            false
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Expect (obligatorio)
    // ─────────────────────────────────────────────────────────────────────────

    pub fn expect(&mut self, expected: &Token, msg: &str) -> Option<SpannedToken> {
        if self.check(expected) {
            let tok = self.current.clone();
            self.advance();
            Some(tok)
        } else {
            self.error(msg);
            None
        }
    }

    /// Expect por patrón (muy importante en lenguajes reales)
    pub fn expect_kind(
        &mut self,
        f: fn(&Token) -> bool,
        msg: &str,
    ) -> Option<SpannedToken> {
        if f(&self.current.token) {
            let tok = self.current.clone();
            self.advance();
            Some(tok)
        } else {
            self.error(msg);
            None
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Errores
    // ─────────────────────────────────────────────────────────────────────────

    fn error(&mut self, msg: &str) {
        let span = self.current.span;
        let full = format!("[ParseError {}] {}", span, msg);
        self.errors.push(full);
    }
}