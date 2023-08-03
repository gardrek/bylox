use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::value::Value;

const STACK_MAX: usize = 256;

#[derive(Default)]
pub struct Vm {
    chunk: Box<Chunk>,
    ip: usize,
    debug: bool,
    stack: Vec<Value>,
}

impl Vm {
    pub fn new(chunk: Box<Chunk>) -> Vm {
        Vm {
            chunk,
            ip: 0,
            debug: true,
            stack: Vec::with_capacity(STACK_MAX),
        }
    }

    pub fn interpret(&mut self, chunk: Box<Chunk>) -> Result<(), InterpretResult> {
        self.chunk = chunk;
        self.ip = 0;
        self.reset_stack();
        self.run()
    }

    pub fn run(&mut self) -> Result<(), InterpretResult> {
        loop {
            if self.debug {
                print!("          ");
                for v in self.stack.iter() {
                    print!("[ {} ]", v);
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }

            let instruction = self.read_byte();
            use OpCode::*;
            match instruction.try_into() {
                Ok(i) => match i {
                    Constant => {
                        let constant_id = self.read_byte() as usize;
                        let constant = self.chunk.get_constant(constant_id);
                        self.push(constant);
                    }
                    LongConstant => {
                        let mut constant_id = 0;
                        for _ in 1..=3 {
                            constant_id = (constant_id << 8) | self.read_byte() as usize;
                        }
                        let constant = self.chunk.get_constant(constant_id);
                        self.push(constant);
                    }
                    Negate => {
                        let v = self.pop();
                        self.push((-f64::from(v)).into());
                    }
                    Return => {
                        println!("{}", self.pop());
                        return Ok(());
                    }
                    Add => self.binary(std::ops::Add::add),
                    Subtract => self.binary(std::ops::Sub::sub),
                    Multiply => self.binary(std::ops::Mul::mul),
                    Divide => self.binary(std::ops::Div::div),
                    Remainder => self.binary(std::ops::Rem::rem),
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

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn binary(&mut self, f: fn(f64, f64) -> f64) {
        let b = self.pop();
        let a = self.pop();
        self.push(f(a.into(), b.into()).into());
    }
}

#[derive(Debug)]
pub enum InterpretResult {
    CompileError,
    RuntimeError,
}

impl std::error::Error for InterpretResult {}

impl std::fmt::Display for InterpretResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use InterpretResult::*;
        match self {
            CompileError => write!(f, "Compile Error"),
            RuntimeError => write!(f, "Runtime Error"),
        }
    }
}
