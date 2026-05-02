use std::iter::Peekable;
use std::str::CharIndices;

/// Define los tipos de tokens disponibles en HULK.
/// El uso de `&'a str` permite un lexer *zero-copy* para rendimiento competitivo.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'a> {
    // Palabras Clave (Keywords)
    Let, In, If, Else, While, For, Function,
    Class, New, Inherits, Is, As, True, False,
    
    // Literales y Nombres
    Identifier(&'a str),
    Number(f64),
    StringLit(&'a str),
    
    // Operadores
    Plus, Minus, Star, Slash, Percent, Caret,
    Assign, Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    And, Or, Not,
    
    // Operadores de HULK específicos
    Arrow,          // =>
    Concat,         // @
    ConcatSpace,    // @@
    
    // Puntuación
    OpenParen, CloseParen,      // ( )
    OpenBrace, CloseBrace,      // { }
    OpenBracket, CloseBracket,  // [ ]
    Comma, Dot, Semicolon, Colon,
    
    // Casos especiales
    Error(&'static str),
}

/// Representa un Token con su tipo y su posición en el código fuente.
#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub start: usize,
    pub end: usize,
}

/// Analizador léxico (Lexer) que procesa una cadena de texto lazy (perezosamente).
/// Implementa `Iterator` para entregar tokens a demanda (O(1) memoria).
pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
        }
    }

    /// Avanza el iterador saltando espacios en blanco.
    fn consume_whitespace(&mut self) {
        while let Some(&(_, c)) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    /// Analiza identificadores y palabras clave.
    fn lex_identifier_or_keyword(&mut self, start: usize) -> Token<'a> {
        let mut end = start;
        while let Some(&(i, c)) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                end = i + c.len_utf8();
                self.chars.next();
            } else {
                break;
            }
        }

        let text = &self.source[start..end];
        let kind = match text {
            "let" => TokenKind::Let,
            "in" => TokenKind::In,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "function" => TokenKind::Function,
            "class" => TokenKind::Class,
            "new" => TokenKind::New,
            "inherits" => TokenKind::Inherits,
            "is" => TokenKind::Is,
            "as" => TokenKind::As,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier(text),
        };

        Token { kind, start, end }
    }

    /// Analiza números enteros y flotantes.
    fn lex_number(&mut self, start: usize) -> Token<'a> {
        let mut end = start;
        let mut has_dot = false;

        while let Some(&(i, c)) = self.chars.peek() {
            if c.is_ascii_digit() {
                end = i + c.len_utf8();
                self.chars.next();
            } else if c == '.' && !has_dot {
                has_dot = true;
                end = i + c.len_utf8();
                self.chars.next();
            } else {
                break;
            }
        }

        let text = &self.source[start..end];
        match text.parse::<f64>() {
            Ok(val) => Token { kind: TokenKind::Number(val), start, end },
            Err(_) => Token { kind: TokenKind::Error("Formato de numero invalido"), start, end }
        }
    }

    /// Analiza literales de cadena (Strings).
    fn lex_string(&mut self, start: usize) -> Token<'a> {
        self.chars.next(); // Consume la comilla inicial
        let mut end = start + 1;
        let mut closed = false;

        while let Some(&(i, c)) = self.chars.peek() {
            end = i + c.len_utf8();
            self.chars.next();
            if c == '"' {
                closed = true;
                break;
            }
        }

        if !closed {
            return Token { kind: TokenKind::Error("Cadena literaria no cerrada"), start, end };
        }

        // El texto interno sin comillas
        let text = &self.source[start + 1..end - 1];
        Token { kind: TokenKind::StringLit(text), start, end }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_whitespace();

        let &(start, c) = self.chars.peek()?;

        // Identificadores y Palabras Clave
        if c.is_alphabetic() || c == '_' {
            return Some(self.lex_identifier_or_keyword(start));
        }

        // Números
        if c.is_ascii_digit() {
            return Some(self.lex_number(start));
        }

        // Cadenas de Texto
        if c == '"' {
            return Some(self.lex_string(start));
        }

        self.chars.next(); // Tomar el caracter principal
        let mut end = start + c.len_utf8();

        // Helper para consumo de tokens de dos caracteres (Peephole)
        let mut match_next = |expected: char| -> bool {
            if let Some(&(i, next_c)) = self.chars.peek() {
                if next_c == expected {
                    end = i + next_c.len_utf8();
                    self.chars.next();
                    return true;
                }
            }
            false
        };

        let kind = match c {
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '^' => TokenKind::Caret,
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            '{' => TokenKind::OpenBrace,
            '}' => TokenKind::CloseBrace,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon,
            '|' => TokenKind::Or,
            '&' => TokenKind::And,
            '!' => if match_next('=') { TokenKind::NotEqual } else { TokenKind::Not },
            '=' => if match_next('=') { TokenKind::Equal } else if match_next('>') { TokenKind::Arrow } else { TokenKind::Assign },
            '<' => if match_next('=') { TokenKind::LessEqual } else { TokenKind::Less },
            '>' => if match_next('=') { TokenKind::GreaterEqual } else { TokenKind::Greater },
            '@' => if match_next('@') { TokenKind::ConcatSpace } else { TokenKind::Concat },
            _ => TokenKind::Error("Caracter no reconocido"),
        };

        Some(Token { kind, start, end })
    }
}
