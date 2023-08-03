#[derive(Default)]
pub struct Scanner {
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: &'a str,
    pub line: usize,
}

impl Scanner {
    pub fn new() -> Scanner {
        let mut scanner = Scanner::default();
        scanner.init();
        scanner
    }

    pub fn init(&mut self) {
        self.start = 0;
        self.current = 0;
        self.line = 1;
    }

    pub fn scan_token<'a>(&mut self, source: &'a str) -> Token<'a> {
        self.start = self.current;

        let kind = if self.is_at_end(source) {
            TokenKind::Eof
        } else {
            let c = self.advance(source);

            use TokenKind::*;

            match c {
                b'(' => LeftParen,
                b')' => RightParen,
                b'{' => LeftBrace,
                b'}' => RightBrace,
                b',' => Comma,
                b'.' => Dot,
                b'-' => Minus,
                b'+' => Plus,
                b';' => Semicolon,
                b'/' => Slash,
                b'*' => Star,
                b'%' => Percent,
                b'!' => {
                    if self.match_advance(source, b'=') {
                        BangEqual
                    } else {
                        Bang
                    }
                }
                b'=' => {
                    if self.match_advance(source, b'=') {
                        EqualEqual
                    } else {
                        Equal
                    }
                }
                b'<' => {
                    if self.match_advance(source, b'=') {
                        LessEqual
                    } else {
                        Less
                    }
                }
                b'>' => {
                    if self.match_advance(source, b'=') {
                        GreaterEqual
                    } else {
                        Greater
                    }
                }
                b'"' => self.string(source),
                b'0'..=b'9' => self.number(source),
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier(source),
                _ => UnexpectedCharacter,
            }
        };

        Token {
            kind,
            span: &source[self.start..self.current],
            line: self.line,
        }
    }

    fn is_at_end(&self, source: &str) -> bool {
        self.current >= source.len()
    }

    fn advance(&mut self, source: &str) -> u8 {
        let i = self.current;
        self.current += 1;
        source.as_bytes()[i]
    }

    fn match_advance(&mut self, source: &str, byte: u8) -> bool {
        if self.is_at_end(source) {
            return false;
        }
        if self.peek(source) != byte {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self, source: &str) -> u8 {
        if self.is_at_end(source) {
            return 0;
        }
        source.as_bytes()[self.current]
    }

    fn peek_next(&self, source: &str) -> u8 {
        if self.is_at_end(source) {
            return 0;
        }
        let i = self.current + 1;
        if i >= source.len() {
            return 0;
        }
        source.as_bytes()[i]
    }

    pub fn skip_whitespace(&mut self, source: &str) {
        loop {
            let c = self.peek(source);

            match c {
                b' ' | b'\r' | b'\t' => {
                    self.advance(source);
                }
                b'\n' => {
                    self.line += 1;
                    self.advance(source);
                }
                b'/' => {
                    if self.peek_next(source) == b'/' {
                        while self.peek(source) != b'\n' && !self.is_at_end(source) {
                            self.advance(source);
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            };
        }
    }

    fn string(&mut self, source: &str) -> TokenKind {
        while self.peek(source) != b'"' && !self.is_at_end(source) {
            if self.peek(source) == b'\n' {
                self.line += 1;
            }
            self.advance(source);
        }

        if self.is_at_end(source) {
            return TokenKind::UnterminatedString;
        }

        self.advance(source);

        TokenKind::String
    }

    fn number(&mut self, source: &str) -> TokenKind {
        while is_digit(self.peek(source)) && !self.is_at_end(source) {
            self.advance(source);
        }

        if self.peek(source) == b'.' && is_digit(self.peek_next(source)) {
            self.advance(source);

            while is_digit(self.peek(source)) && !self.is_at_end(source) {
                self.advance(source);
            }
        }

        TokenKind::Number
    }

    fn identifier(&mut self, source: &str) -> TokenKind {
        while is_alpha(self.peek(source)) || is_digit(self.peek(source)) {
            self.advance(source);
        }

        self.identifier_type(source)
    }

    fn identifier_type(&mut self, source: &str) -> TokenKind {
        match str_to_keyword(&source[self.start..self.current]) {
            Some(keyword) => keyword,
            None => TokenKind::Identifier,
        }
    }
}

fn is_digit(byte: u8) -> bool {
    byte.is_ascii_digit()
}

fn is_alpha(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_uppercase() || byte == b'_'
}

fn str_to_keyword(s: &str) -> Option<TokenKind> {
    use TokenKind::*;
    Some(match s {
        "and" => And,
        "class" => Class,
        "else" => Else,
        "false" => False,
        "for" => For,
        "fun" => Fun,
        "if" => If,
        "nil" => Nil,
        "or" => Or,
        "print" => Print,
        "return" => Return,
        "super" => Super,
        "this" => This,
        "true" => True,
        "var" => Var,
        "while" => While,
        _ => return None,
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokenKind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Percent,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    // Other
    UnterminatedString,
    UnexpectedCharacter,
    Eof,
}
