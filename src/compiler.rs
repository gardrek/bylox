use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::scanner::Scanner;
use crate::scanner::Token;
use crate::scanner::TokenKind;
use crate::value::Value;
use crate::vm::InterpretError;

use derive_try_from_primitive::TryFromPrimitive;

struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Option<Token<'a>>,
    previous: Option<Token<'a>>,
    had_error: bool,
    panic_mode: bool,
    chunk: Chunk,
}

type ParseFn = fn(&mut Parser<'_>) -> ();

/*
struct Rule {
    prefix: ParseFn,
    infix: ParseFn,
    precedence: Precedence,
}
*/

#[repr(u8)]
#[derive(TryFromPrimitive, Clone, Copy)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: Scanner<'a>) -> Parser<'a> {
        Parser {
            scanner,
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
            chunk: Chunk::default(),
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            self.current = Some(self.scanner.scan_token());

            match &self.current {
                Some(t) => match t.kind {
                    TokenKind::UnterminatedString => {
                        self.report_error_at_current("unterminated string")
                    }
                    TokenKind::UnexpectedCharacter => {
                        self.report_error_at_current("unexpected character")
                    }
                    _ => break,
                },
                None => todo!(),
            }
        }
    }

    // errorAtCurrent() in book
    fn report_error_at_current(&mut self, message: &str) {
        self.report_error_at(self.current.as_ref().unwrap().clone(), message)
    }

    // error() in book
    fn report_error_at_previous(&mut self, message: &str) {
        self.report_error_at(self.previous.as_ref().unwrap().clone(), message)
    }

    // errorAt() in book
    fn report_error_at(&mut self, token: Token<'_>, message: &str) {
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;

        eprint!("[line {}] Error", token.line);

        match token.kind {
            TokenKind::UnterminatedString | TokenKind::UnexpectedCharacter => print!(""),
            TokenKind::Eof => eprint!(" at end"),
            _ => eprint!(" at `{}`", token.span),
        }

        eprintln!(": {}", message);
        self.had_error = true;
    }

    fn consume(&mut self, kind: TokenKind, message: &str) {
        if self.current.as_ref().unwrap().kind == kind {
            self.advance();
            return;
        }

        self.report_error_at_current(message);
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write(byte, self.previous.as_ref().unwrap().line)
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.chunk
                .write(*byte, self.previous.as_ref().unwrap().line)
        }
    }

    fn emit_constant(&mut self, value: Value) {
        self.chunk
            .write_constant(value, self.previous.as_ref().unwrap().line);
    }
}

// parsing rules
impl Parser<'_> {
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let prefix_rule = get_rule_prefix(&self.previous.as_ref().unwrap().kind);

        if prefix_rule.is_none() {
            self.report_error_at_previous("Expect expression.");
            return;
        }

        prefix_rule.unwrap()(self);

        while precedence as u8 <= get_rule_precedence(&self.current.as_ref().unwrap().kind) as u8 {
            self.advance();
            let infix_rule = get_rule_infix(&self.previous.as_ref().unwrap().kind);
            infix_rule.unwrap()(self);
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expect `)`.");
    }

    fn number(&mut self) {
        let string = self.previous.as_ref().unwrap().span;
        let number = string.parse::<f64>().unwrap();
        self.emit_constant(number.into());
    }

    fn unary(&mut self) {
        let operator_kind = self.previous.as_ref().unwrap().kind;

        self.parse_precedence(Precedence::Assignment);

        match operator_kind {
            TokenKind::Bang => self.emit_byte(OpCode::Not as u8),
            TokenKind::Minus => self.emit_byte(OpCode::Negate as u8),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self) {
        let operator_kind = self.previous.as_ref().unwrap().kind;

        let precedence = get_rule_precedence(&operator_kind);

        self.parse_precedence((precedence as u8 + 1).try_into().unwrap());

        match operator_kind {
            TokenKind::BangEqual => self.emit_bytes(&[OpCode::Equal as u8, OpCode::Not as u8]),
            TokenKind::EqualEqual => self.emit_byte(OpCode::Equal as u8),
            TokenKind::Greater => self.emit_byte(OpCode::Greater as u8),
            TokenKind::GreaterEqual => self.emit_bytes(&[OpCode::Less as u8, OpCode::Not as u8]),
            TokenKind::Less => self.emit_byte(OpCode::Less as u8),
            TokenKind::LessEqual => self.emit_bytes(&[OpCode::Greater as u8, OpCode::Not as u8]),
            TokenKind::Plus => self.emit_byte(OpCode::Add as u8),
            TokenKind::Minus => self.emit_byte(OpCode::Subtract as u8),
            TokenKind::Star => self.emit_byte(OpCode::Multiply as u8),
            TokenKind::Slash => self.emit_byte(OpCode::Divide as u8),
            TokenKind::Percent => self.emit_byte(OpCode::Remainder as u8),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self) {
        let kind = self.previous.as_ref().unwrap().kind;

        match kind {
            TokenKind::Nil => self.emit_byte(OpCode::Nil as u8),
            TokenKind::True => self.emit_byte(OpCode::True as u8),
            TokenKind::False => self.emit_byte(OpCode::False as u8),
            _ => unreachable!(),
        }
    }

    fn string(&mut self) {
        let token = self.previous.as_ref().unwrap();

        match parse_string(&token.span[1..(token.span.len() - 1)]) {
            Ok(s) => self.emit_constant(s.into()),
            Err(s) => self.report_error_at_previous(s),
        }
    }
}

fn get_rule_prefix(token: &TokenKind) -> Option<ParseFn> {
    let literal = |a: &mut Parser<'_>| Parser::literal(a);
    Some(match token {
        TokenKind::LeftParen => |a: &mut Parser<'_>| Parser::grouping(a),
        TokenKind::Minus => |a: &mut Parser<'_>| Parser::unary(a),
        TokenKind::Bang => |a: &mut Parser<'_>| Parser::unary(a),
        TokenKind::Number => |a: &mut Parser<'_>| Parser::number(a),
        TokenKind::Nil => literal,
        TokenKind::False => literal,
        TokenKind::True => literal,
        TokenKind::String => |a: &mut Parser<'_>| Parser::string(a),
        _ => return None,
    })
}

fn get_rule_infix(token: &TokenKind) -> Option<ParseFn> {
    let binary = |a: &mut Parser<'_>| Parser::binary(a);
    Some(match token {
        TokenKind::Plus => binary,
        TokenKind::Minus => binary,
        TokenKind::Star => binary,
        TokenKind::Slash => binary,
        TokenKind::Percent => binary,
        TokenKind::EqualEqual => binary,
        TokenKind::Greater => binary,
        TokenKind::GreaterEqual => binary,
        TokenKind::Less => binary,
        TokenKind::LessEqual => binary,
        _ => return None,
    })
}

fn get_rule_precedence(token: &TokenKind) -> Precedence {
    match token {
        TokenKind::Plus => Precedence::Term,
        TokenKind::Minus => Precedence::Term,
        TokenKind::Star => Precedence::Factor,
        TokenKind::Slash => Precedence::Factor,
        TokenKind::Percent => Precedence::Factor,
        TokenKind::BangEqual => Precedence::Equality,
        TokenKind::EqualEqual => Precedence::Equality,
        TokenKind::Greater => Precedence::Comparison,
        TokenKind::GreaterEqual => Precedence::Comparison,
        TokenKind::Less => Precedence::Comparison,
        TokenKind::LessEqual => Precedence::Comparison,
        _ => Precedence::None,
    }
}

fn parse_string(escaped_string: &str) -> Result<String, &'static str> {
    let mut final_string = String::with_capacity(escaped_string.len());
    let mut iter = escaped_string.chars();

    while let Some(ch) = iter.next() {
        final_string.push(match ch {
            '\\' => match iter.next() {
                Some(esc) => match esc {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '0' => '\0',
                    '\'' => '\'',
                    '\"' => '\"',
                    'x' => return Err("unimplemented string escape `\\x`"),
                    'u' => return Err("unimplemented string escape `\\u`"),
                    _ => return Err("unrecognized string escape"),
                },
                None => return Err("unexpected end of string"),
            },
            _ => ch,
        });
    }

    Ok(final_string)
}

pub fn compile(source: &str) -> Result<Chunk, InterpretError> {
    let scanner = Scanner::new(source);

    let mut parser = Parser::new(scanner);

    parser.advance();
    parser.expression();
    parser.consume(TokenKind::Eof, "Expect end of expression.");

    if parser.had_error {
        return Err(InterpretError::CompileError("compile error"));
    }

    parser.emit_byte(OpCode::Return as u8);

    if !parser.had_error {
        parser.chunk.disassemble("code");
    }

    Ok(parser.chunk)
}
