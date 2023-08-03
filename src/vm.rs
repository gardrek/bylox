use crate::chunk::Chunk;
use crate::chunk::OpCode;

pub struct Vm {
    chunk: Box<Chunk>,
    ip: usize,
    debug: bool,
}

#[derive(Debug)]
pub enum InterpretResult {
    CompileError,
    RuntimeError,
}

impl Vm {
    pub fn new(chunk: Box<Chunk>) -> Vm {
        Vm {
            chunk,
            ip: 0,
            debug: true,
        }
    }

    pub fn interpret(&mut self, chunk: Box<Chunk>) -> Result<(), InterpretResult> {
        self.chunk = chunk;
        self.ip = 0;
        self.run()
    }

    pub fn run(&mut self) -> Result<(), InterpretResult> {
        loop {
            if self.debug { self.chunk.disassemble_instruction(self.ip); }
            let instruction = self.read_byte();
            match instruction.try_into() {
                Ok(i) => match i {
                    OpCode::Return => return Ok(()),
                    OpCode::Constant => {
                        let constant_id = self.read_byte() as usize;
                        let constant = self.chunk.get_constant(constant_id);
                        println!("{}", constant);
                    }
                    OpCode::LongConstant => {
                        let mut constant_id = 0;
                        for _ in 1..=3 {
                            constant_id = (constant_id << 8) | self.read_byte() as usize;
                        }
                        let constant = self.chunk.get_constant(constant_id);
                        println!("{}", constant);
                    }
                },
                _ => {
                    panic!("Unknown opcode {}", instruction);
                }
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.read(self.ip);
        self.ip += 1;
        byte
    }
}
