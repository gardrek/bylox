use crate::chunk::Chunk;
use crate::scanner::Scanner;
use crate::scanner::TokenKind;
use crate::vm::InterpretResult;

pub fn compile(source: &str) -> Result<Chunk, InterpretResult> {
    let mut scanner = Scanner::new();

    //~ let source = source.to_string();

    let mut line = 0;

    loop {
        scanner.skip_whitespace(source);
        let token = scanner.scan_token(source);

        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }
        println!("{:?} `{}`", token.kind, token.span);

        if token.kind == TokenKind::Eof {
            break;
        }
    }

    todo!()
}
