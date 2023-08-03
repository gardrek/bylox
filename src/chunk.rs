use crate::debug::LineMap;
use crate::value::Value;

use derive_try_from_primitive::TryFromPrimitive;

#[repr(u8)]
#[derive(TryFromPrimitive)]
pub enum OpCode {
    Constant,
    LongConstant,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Return,
}

#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    lines: LineMap,
}

impl Chunk {
    pub fn read(&self, index: usize) -> u8 {
        self.code[index]
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        let offset = self.code.len();
        self.code.push(byte);
        self.lines.add(offset, line);
    }

    pub fn write_constant(&mut self, constant: Value, line: usize) {
        let id = self.add_constant(constant);
        match id {
            0..=0xff => {
                let id = id as u8;
                self.write(OpCode::Constant as u8, line);
                self.write(id, line);
            }
            0x100..=0xffffff => {
                let id = id as u32;
                let top_byte = OpCode::LongConstant as u8;
                let mut bytes = id.to_be_bytes();
                bytes[0] = top_byte;
                for b in bytes {
                    self.write(b, line);
                }
            }
            _ => panic!("reached constant limit fo 2^24"),
        }
    }

    fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn get_constant(&self, id: usize) -> Value {
        self.constants[id]
    }
}

impl Chunk {
    fn get_line(&self, offset: usize) -> usize {
        self.lines.get_line(offset)
    }

    pub fn disassemble(&self, name: &str) {
        println!("=== {} ===", name);

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        let line = self.get_line(offset);

        if offset > 0 && self.get_line(offset - 1) == line {
            print!("   | ");
        } else {
            print!("{:04} ", line);
        }

        fn simple_instruction(name: &str, offset: usize) -> usize {
            println!("{}", name);
            offset + 1
        }

        let instruction = self.code[offset];
        use OpCode::*;
        match instruction.try_into() {
            Ok(i) => match i {
                Constant => self.constant_instruction("OP_CONSTANT", offset),
                LongConstant => self.long_constant_instruction("OP_CONSTANT_LONG", offset),
                Negate => simple_instruction("OP_NEGATE", offset),
                Return => simple_instruction("OP_RETURN", offset),
                Add => simple_instruction("OP_ADD", offset),
                Subtract => simple_instruction("OP_SUBTRACT", offset),
                Multiply => simple_instruction("OP_MULTIPLY", offset),
                Divide => simple_instruction("OP_DIVIDE", offset),
                Remainder => simple_instruction("OP_REMAINDER", offset),
            },
            _ => {
                println!("Unknown opcode {}", instruction);
                offset + 1
            }
        }
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        let value = self.constants[constant as usize];
        println!("{:16} {} {}", name, constant, value);
        offset + 2
    }

    fn long_constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant_bytes = &self.code[offset..(offset + 4)];
        let constant = u32::from_be_bytes(constant_bytes.try_into().unwrap()) & 0xffffff;
        let value = self.constants[constant as usize];
        println!("{:16} {} {}", name, constant, value);
        offset + 4
    }
}
