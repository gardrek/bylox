#[derive(Default)]
pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: &'a str,
    pub line: usize,
}

impl<'s> Scanner<'s> {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token<'s> {
        self.skip_whitespace();

        self.start = self.current;

        let kind = if self.is_at_end() {
            TokenKind::Eof
        } else {
            let c = self.advance();

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
                    if self.match_advance(b'=') {
                        BangEqual
                    } else {
                        Bang
                    }
                }
                b'=' => {
                    if self.match_advance(b'=') {
                        EqualEqual
                    } else {
                        Equal
                    }
                }
                b'<' => {
                    if self.match_advance(b'=') {
                        LessEqual
                    } else {
                        Less
                    }
                }
                b'>' => {
                    if self.match_advance(b'=') {
                        GreaterEqual
                    } else {
                        Greater
                    }
                }
                b'"' => self.string(),
                b'0'..=b'9' => self.number(),
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier(),
                _ => UnexpectedCharacter,
            }
        };

        Token {
            kind,
            span: &self.source[self.start..self.current],
            line: self.line,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> u8 {
        let i = self.current;
        self.current += 1;
        self.source.as_bytes()[i]
    }

    fn match_advance(&mut self, byte: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() != byte {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            return 0;
        }
        self.source.as_bytes()[self.current]
    }

    fn peek_next(&self) -> u8 {
        if self.is_at_end() {
            return 0;
        }
        let i = self.current + 1;
        if i >= self.source.len() {
            return 0;
        }
        self.source.as_bytes()[i]
    }

    pub fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();

            match c {
                b' ' | b'\r' | b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'/' => {
                    if self.peek_next() == b'/' {
                        while self.peek() != b'\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            };
        }
    }

    fn string(&mut self) -> TokenKind {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            if self.peek() == b'\\' {
                // advance to skip the check for double quote
                self.advance();

                if self.is_at_end() {
                    return TokenKind::UnterminatedString;
                }
            }
            self.advance();
        }

        if self.is_at_end() {
            return TokenKind::UnterminatedString;
        }

        self.advance();

        TokenKind::String
    }

    fn number(&mut self) -> TokenKind {
        while is_digit(self.peek()) && !self.is_at_end() {
            self.advance();
        }

        if self.peek() == b'.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) && !self.is_at_end() {
                self.advance();
            }
        }

        TokenKind::Number
    }

    fn identifier(&mut self) -> TokenKind {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }

        self.identifier_type()
    }

    fn identifier_type(&mut self) -> TokenKind {
        match str_to_keyword(&self.source[self.start..self.current]) {
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
