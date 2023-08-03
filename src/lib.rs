mod chunk;
mod debug;
mod value;
mod vm;

use chunk::Chunk;
use chunk::OpCode;

pub fn run() {
    let mut chunk = Chunk::default();
    let line = 123;
    for i in 0..=256 {
        chunk.write_constant((i * 1000).into(), line);
    }
    chunk.write(OpCode::Return as u8, line);
    chunk.write(OpCode::Return as u8, line + 3);
    chunk.write(OpCode::Return as u8, line + 4);
    chunk.disassemble("test chunk");

    let mut vm = vm::Vm::new(Box::new(chunk));

    vm.run().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        run();
    }
}
