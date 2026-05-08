use crate::lexer::lexer::{Token, SpannedToken, TokenStream};
use crate::parser::ast::{BinaryOp, BuiltinFn, ConstValue, Expression, Factor, FunctionBody, FunctionDef, FunctionParam, Statement, Term, UnaryOp};
use std::collections::HashMap;


// ---------------------------------------------
// Parser
// ---------------------------------------------

pub struct Parser<'src> {
    tokens: TokenStream<'src>,
    current: SpannedToken,
    pub errors: Vec<String>,
}

impl<'src> Parser<'src> {

    // ---------------------------------------------
    // Initialization
    // ---------------------------------------------

    pub fn new(mut tokens: TokenStream<'src>) -> Self {
        let first = tokens.next_token();

        Self {
            tokens,
            current: first,
            errors: Vec::new(),
        }
    }


    // ---------------------------------------------
    // Core navigation
    // ---------------------------------------------

    /// Advance to the next token.
    pub fn advance(&mut self) {
        self.current = self.tokens.next_token();
    }

    /// Current token without consuming it.
    pub fn peek(&self) -> &Token {
        &self.current.token
    }

    /// Return true at EOF.
    pub fn is_at_end(&self) -> bool {
        self.current.token == Token::Eof
    }

    // ---------------------------------------------
    // Basic matching
    // ---------------------------------------------

    /// Check exact token match.
    pub fn check(&self, token: &Token) -> bool {
        &self.current.token == token
    }

    /// Consume the token if it matches exactly.
    pub fn matches(&mut self, expected: &Token) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume one of several tokens and return it.
    pub fn match_any(&mut self, tokens: &[Token]) -> Option<SpannedToken> {
        for t in tokens {
            if &self.current.token == t {
                let tok = self.current.clone();
                self.advance();
                return Some(tok);
            }
        }
        None
    }

    // ---------------------------------------------
    // Kind matching
    // ---------------------------------------------

    /// Check by predicate.
    pub fn check_kind(&self, f: fn(&Token) -> bool) -> bool {
        f(&self.current.token)
    }

    /// Consume if the predicate matches.
    pub fn match_kind(&mut self, f: fn(&Token) -> bool) -> bool {
        if f(&self.current.token) {
            self.advance();
            true
        } else {
            false
        }
    }

    // ---------------------------------------------
    // Expect and errors
    // ---------------------------------------------

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

    /// Expect by predicate.
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

    /// Record a parsing error with the current token's span.
    fn error(&mut self, msg: &str) {
        let span = self.current.span;
        let full = format!("[ParseError {}] {}", span, msg);
        self.errors.push(full);
    }

    /// Synchronize to a safe point for error recovery (panic-mode).
    /// Skips tokens until a likely statement/declaration boundary is found.
    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            match self.peek() {
                // Recovery points: semicolon, closing brace, EOF, or start of new statement/declaration
                Token::Semicolon | Token::RBrace | Token::Eof | Token::Function | Token::Let => {
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    // ---------------------------------------------
    // Function parsing
    // ---------------------------------------------

    /// Parse a function definition, which starts with the 'function' keyword, followed by the name, parameters, and body.
    pub fn parse_function(&mut self) -> Option<FunctionDef> {
        self.expect(&Token::Function, "se esperaba 'function'")?;

        let name = self.parse_identifier("se esperaba el nombre de la funcion")?;
        let params = self.parse_function_params()?;

        let body = if self.matches(&Token::Arrow) {
            self.parse_function_inline_body()?
        } else if self.check(&Token::LBrace) {
            self.parse_function_block_body()?
        } else {
            self.error("se esperaba '=>' o '{' en el cuerpo de la funcion");
            return None;
        };

        Some(FunctionDef { name, params, body })
    }

    /// Parse the parameter list of a function, which is enclosed in parentheses and separated by commas.
    fn parse_function_params(&mut self) -> Option<Vec<FunctionParam>> {
        self.expect(&Token::LParen, "se esperaba '(' despues del nombre de la funcion")?;

        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                params.push(self.parse_function_param()?);

                if self.matches(&Token::Comma) {
                    continue;
                }

                break;
            }
        }

        self.expect(&Token::RParen, "se esperaba ')' al cerrar la lista de parametros")?;
        Some(params)
    }

    /// Parse a single function parameter, which consists of an identifier and an optional type annotation.
    fn parse_function_param(&mut self) -> Option<FunctionParam> {
        let name = self.parse_identifier("se esperaba un nombre de parametro")?;
        let ty = if self.matches(&Token::Colon) {
            Some(self.parse_type_name("se esperaba un tipo de parametro")?)
        } else {
            None
        };

        Some(FunctionParam { name, ty })
    }

    /// Parse an inline function body, which is a single expression followed by a semicolon.
    fn parse_function_inline_body(&mut self) -> Option<FunctionBody> {
        let expr = self.parse_expr()?;
        self.expect(&Token::Semicolon, "se esperaba ';' al final del cuerpo inline")?;
        Some(FunctionBody::Inline(expr))
    }

    /// Parse a block function body, which is a series of expressions enclosed in braces and separated by semicolons.
    fn parse_function_block_body(&mut self) -> Option<FunctionBody> {
        self.expect(&Token::LBrace, "se esperaba '{' para abrir el cuerpo de la funcion")?;

        let mut expressions = Vec::new();
        if self.check(&Token::RBrace) {
            self.error("el cuerpo de la funcion no puede estar vacio");
            return None;
        }

        loop {
            let expr = self.parse_expr()?;
            expressions.push(expr);

            if self.matches(&Token::Semicolon) {
                if self.check(&Token::RBrace) {
                    break;
                }
                continue;
            }

            break;
        }

        self.expect(&Token::RBrace, "se esperaba '}' al cerrar el cuerpo de la funcion")?;
        Some(FunctionBody::Block(expressions))
    }

    /// Parse an expression, which can be a term or a binary operation of terms.
    fn parse_identifier(&mut self, msg: &str) -> Option<SpannedToken> {
        match self.peek() {
            Token::Ident(_) | Token::InternalIdent(_) => {
                let tok = self.current.clone();
                self.advance();
                Some(tok)
            }
            _ => {
                self.error(msg);
                None
            }
        }
    }

    /// Parse a type name, which can be an identifier or a built-in type keyword.
    fn parse_type_name(&mut self, msg: &str) -> Option<SpannedToken> {
        match self.peek() {
            Token::Ident(name) | Token::InternalIdent(name) => {
                let _ = name;
                let tok = self.current.clone();
                self.advance();
                Some(tok)
            }
            Token::TypNumber => {
                let tok = self.current.clone();
                self.advance();
                Some(tok)
            }
            Token::TypString => {
                let tok = self.current.clone();
                self.advance();
                Some(tok)
            }
            Token::TypBool => {
                let tok = self.current.clone();
                self.advance();
                Some(tok)
            }
            _ => {
                self.error(msg);
                None
            }
        }
    }


    // ---------------------------------------------
    // Expression parsing
    // ---------------------------------------------

    /// Parse an expression with correct operator precedence (left-associative for +/-).
    pub fn parse_expr(&mut self) -> Option<Expression> {
        let mut left = Expression::Term(self.parse_term()?);

        // Loop for left-associativity: a + b + c = ((a + b) + c)
        while let Some(op_tok) = self.match_any(&[Token::Plus, Token::Minus]) {
            let op = Self::binary_op_from_token(&op_tok.token)?;
            let right = Expression::Term(self.parse_term()?);
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    /// Parse a let statement: let x = expr, y = expr in body_expr
    pub fn parse_let(&mut self) -> Option<Statement> {
        self.expect(&Token::Let, "se esperaba 'let'")?;

        // Parse assignments: x = expr, y = expr, ...
        let mut assignments = HashMap::new();

        loop {
            let name = self.parse_identifier("se esperaba nombre de variable en asignación")?;
            self.expect(&Token::Eq, "se esperaba '=' en asignación")?;
            let expr = self.parse_expr()?;

            assignments.insert(name, expr);

            if self.matches(&Token::Comma) {
                continue;
            }

            break;
        }

        // Expect 'in'
        self.expect(&Token::In, "se esperaba 'in' después de las asignaciones")?;

        // Parse body expression
        let body = Box::new(self.parse_expr()?);

        let _ = self.matches(&Token::Semicolon);

        Some(Statement::Assign {
            assignments,
            body,
        })
    }

    /// Parse a term with left-associativity for * and /.
    pub fn parse_term(&mut self) -> Option<Term> {
        let mut left = Term::Factor(self.parse_power()?);

        // Loop for left-associativity: a * b * c = ((a * b) * c)
        while let Some(op_tok) = self.match_any(&[Token::Star, Token::Slash]) {
            let op = Self::binary_op_from_token(&op_tok.token)?;
            let right = Term::Factor(self.parse_power()?);
            left = Term::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    /// Parse power (exponentiation) with right-associativity.
    /// Power has higher precedence than multiplication/division.
    /// NOTE: Current implementation does NOT handle ^. This requires AST redesign
    /// to represent power without Factor::Binary. For now, ^ will produce an error.
    /// TODO: Add Factor::Power node or refactor to use Term level for power.
    fn parse_power(&mut self) -> Option<Factor> {
        self.parse_unary()
    }

    /// Parse unary expressions (prefix operators like -, +).
    fn parse_unary(&mut self) -> Option<Factor> {
        if let Some(op_tok) = self.match_any(&[Token::Minus, Token::Plus]) {
            let op = match op_tok.token {
                Token::Minus => UnaryOp::Neg,
                Token::Plus => UnaryOp::Pos,
                _ => unreachable!(),
            };
            let operand = Box::new(self.parse_unary()?);
            return Some(Factor::Unary { op, operand });
        }

        self.parse_factor()
    }

    /// Parse a factor (primary expression): number, ident, grouped expr, call, builtin, constant.
    pub fn parse_factor(&mut self) -> Option<Factor> {
        let base = match self.peek() {
            Token::Number(value) => {
                let value = value.clone();
                self.advance();
                Factor::Number(value)
            }

            Token::Ident(name) | Token::InternalIdent(name) => {
                let name = name.clone();
                self.advance();
                
                // Check if this is a function call: Ident followed by (
                if self.check(&Token::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(&Token::RParen) {
                        loop {
                            let expr = self.parse_expr()?;
                            args.push(expr);
                            if self.matches(&Token::Comma) {
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(&Token::RParen, "se esperaba ')' al cerrar llamada")?;
                    Factor::Call { callee: name, args }
                } else {
                    Factor::Ident(name)
                }
            }

            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen, "se esperaba ')' al cerrar expresion")?;
                Factor::Group(Box::new(expr))
            }

            Token::Sin => {
                self.advance();
                let args = self.parse_call_args("sin")?;
                Factor::BuiltinCall { func: BuiltinFn::Sin, args }
            }
            Token::Cos => {
                self.advance();
                let args = self.parse_call_args("cos")?;
                Factor::BuiltinCall { func: BuiltinFn::Cos, args }
            }
            Token::Log => {
                self.advance();
                let args = self.parse_call_args("log")?;
                Factor::BuiltinCall { func: BuiltinFn::Log, args }
            }
            Token::Sqrt => {
                self.advance();
                let args = self.parse_call_args("sqrt")?;
                Factor::BuiltinCall { func: BuiltinFn::Sqrt, args }
            }
            Token::Exp => {
                self.advance();
                let args = self.parse_call_args("exp")?;
                Factor::BuiltinCall { func: BuiltinFn::Exp, args }
            }
            Token::Rand => {
                self.advance();
                let args = self.parse_call_args("rand")?;
                Factor::BuiltinCall { func: BuiltinFn::Rand, args }
            }
            Token::Pi => {
                self.advance();
                Factor::Const(ConstValue::Pi)
            }
            Token::E => {
                self.advance();
                Factor::Const(ConstValue::E)
            }
            Token::True => {
                self.advance();
                Factor::Const(ConstValue::True)
            }
            Token::False => {
                self.advance();
                Factor::Const(ConstValue::False)
            }

            _ => {
                self.error("se esperaba un factor");
                return None;
            }
        };

        Some(base)
    }

    /// Parse arguments for a function call, expecting them to be enclosed in parentheses.
    fn parse_call_args(&mut self, ctx: &str) -> Option<Vec<Expression>> {
        let msg = format!("se esperaba '(' despues de {}", ctx);
        self.expect(&Token::LParen, &msg)?;

        let mut args = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                let expr = self.parse_expr()?;
                args.push(expr);

                if self.matches(&Token::Comma) {
                    continue;
                }

                break;
            }
        }

        self.expect(&Token::RParen, "se esperaba ')' al cerrar llamada")?;
        Some(args)
    }

    /// Convert a token to a binary operator if it matches.
    fn binary_op_from_token(token: &Token) -> Option<BinaryOp> {
        match token {
            Token::Plus => Some(BinaryOp::Add),
            Token::Minus => Some(BinaryOp::Sub),
            Token::Star => Some(BinaryOp::Mul),
            Token::Slash => Some(BinaryOp::Div),
            Token::Caret => Some(BinaryOp::Pow),
            _ => None,
        }
    }

    // =====================================================
    // PUBLIC ENTRY POINT
    // =====================================================

    /// Parse a program: a sequence of top-level declarations (functions, statements).
    /// Returns a vector of FunctionDef and consumes tokens until EOF.
    /// Accumulates all errors encountered and continues parsing on error.
    pub fn parse_program(&mut self) -> (Vec<FunctionDef>, Vec<String>) {
        let mut functions = Vec::new();
        let mut all_errors = Vec::new();

        while !self.is_at_end() {
            match self.peek() {
                Token::Function => {
                    // Parse function declaration
                    if let Some(func) = self.parse_function() {
                        functions.push(func);
                    } else {
                        all_errors.extend(self.errors.drain(..));
                        self.synchronize();
                    }
                }
                Token::Eof => {
                    break;
                }
                _ => {
                    self.error("se esperaba 'function' o fin de archivo");
                    all_errors.extend(self.errors.drain(..));
                    self.synchronize();
                }
            }
        }

        all_errors.extend(self.errors.drain(..));
        (functions, all_errors)
    }
}
