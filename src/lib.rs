mod compiler;
mod debug;
mod scanner;
//~ mod table;
mod value;

pub mod arg;
pub mod chunk;
pub mod vm;

pub use compiler::compile;

use chunk::Chunk;
use chunk::OpCode;

pub fn run_file(path: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    Ok(run_string(&source)?)
}

pub fn run_string(source: &str) -> Result<(), vm::InterpretError> {
    let mut vm = vm::Vm::new(Box::default());

    vm.interpret(source)
}

pub fn run_test() {
    let mut chunk = Chunk::default();

    /*
    let line = 123;
    for i in 0..=256 {
        chunk.write_constant((i * 1000 + 100).into(), line + i as usize);
        chunk.write(OpCode::Negate as u8, line + i as usize);
        chunk.write(OpCode::Return as u8, line + i as usize);
    }
    chunk.write(OpCode::Return as u8, line);
    chunk.write(OpCode::Return as u8, line + 3);
    chunk.write(OpCode::Return as u8, line + 4);
    chunk.disassemble("test chunk");
    */

    chunk.write_constant(1.2.into(), 1);
    chunk.write_constant(3.4.into(), 1);
    chunk.write(OpCode::Add as u8, 1);
    chunk.write_constant(3.4.into(), 1);
    chunk.write(OpCode::Divide as u8, 1);
    chunk.write(OpCode::Negate as u8, 1);
    chunk.write(OpCode::Return as u8, 1);

    let mut vm = vm::Vm::new(Box::new(chunk));

    vm.run().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        run_test();
    }
}
