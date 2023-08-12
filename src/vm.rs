use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::compiler::compile;
use crate::value::Object;
use crate::value::Value;

use std::collections::HashMap;
use std::rc::Rc;

const STACK_MAX: usize = 256;

#[derive(Default)]
pub struct Vm {
    chunk: Box<Chunk>,
    ip: usize,
    debug: bool,
    stack: Vec<Value>,
    objects: Vec<std::rc::Weak<Object>>,
    globals: HashMap<String, Value>,
}

impl Vm {
    pub fn new(chunk: Box<Chunk>) -> Vm {
        Vm {
            chunk,
            ip: 0,
            debug: true,
            stack: Vec::with_capacity(STACK_MAX),
            objects: vec![],
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretError> {
        self.chunk = Box::new(compile(source)?);
        self.ip = 0;
        //~ self.reset_stack(); // do we reset here or what? who knows
        self.run()
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            if self.debug {
                print!("    stack ");
                for v in self.stack.iter() {
                    print!("[ {:?} ]", v);
                }
                println!();
                print!("  objects ");
                for v in self.objects.iter() {
                    match v.upgrade() {
                        Some(p) => print!("[ Object#{:p} ]", p),
                        None => print!("[ Empty ]"),
                    }
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }

            let instruction = self.read_byte();
            use OpCode::*;
            match instruction.try_into() {
                Ok(inst) => match inst {
                    Constant => {
                        let constant_id = self.read_byte() as usize;
                        let constant = self.chunk.get_constant(constant_id);
                        self.push(constant.clone());
                    }
                    LongConstant => {
                        let constant_id = self.read_int(3);
                        let constant = self.chunk.get_constant(constant_id);
                        self.push(constant.clone());
                    }
                    Nil => self.push(Value::Nil),
                    True => self.push(Value::Boolean(true)),
                    False => self.push(Value::Boolean(false)),
                    Pop => {
                        self.pop()?;
                    }
                    GetGlobal => {
                        let constant_id = self.read_byte();
                        let constant = self.chunk.get_constant(constant_id as usize);
                        let name = constant.as_string().unwrap().to_string();
                        if let Some(value) = self.globals.get(&name) {
                            self.push(value.clone());
                        } else {
                            let e = self.report_runtime_error("Undefined variable.");
                            return Err(e);
                        }
                    }
                    GetLongGlobal => {
                        let constant_id = self.read_int(3);
                        let constant = self.chunk.get_constant(constant_id);
                        let name = constant.as_string().unwrap().to_string();
                        if let Some(value) = self.globals.get(&name) {
                            self.push(value.clone());
                        } else {
                            let e = self.report_runtime_error("Undefined variable.");
                            return Err(e);
                        }
                    }
                    DefineGlobal => {
                        let constant_id = self.read_byte();
                        let constant = self.chunk.get_constant(constant_id as usize);
                        let name = constant.as_string().unwrap().to_string();
                        self.globals.insert(name, self.peek(0).clone());
                        self.pop()?;
                    }
                    DefineLongGlobal => {
                        let constant_id = self.read_int(3);
                        let constant = self.chunk.get_constant(constant_id);
                        let name = constant.as_string().unwrap().to_string();
                        self.globals.insert(name, self.peek(0).clone());
                        self.pop()?;
                    }
                    SetGlobal => {
                        let constant_id = self.read_byte();
                        let constant = self.chunk.get_constant(constant_id as usize);
                        let name = constant.as_string().unwrap().to_string();
                        let val = self.peek(0).clone();
                        if let std::collections::hash_map::Entry::Occupied(mut entry) =
                            self.globals.entry(name)
                        {
                            entry.insert(val);
                        } else {
                            let e = self.report_runtime_error("Undefined variable.");
                            return Err(e);
                        }
                    }
                    SetLongGlobal => {
                        let constant_id = self.read_int(3);
                        let constant = self.chunk.get_constant(constant_id);
                        let name = constant.as_string().unwrap().to_string();
                        let val = self.peek(0).clone();
                        if let std::collections::hash_map::Entry::Occupied(mut entry) =
                            self.globals.entry(name)
                        {
                            entry.insert(val);
                        } else {
                            let e = self.report_runtime_error("Undefined variable.");
                            return Err(e);
                        }
                    }
                    Equal => {
                        let b = self.pop()?;
                        let a = self.pop()?;
                        self.push(Value::Boolean(a == b));
                    }
                    Greater => self.binary_cmp(greater_than)?,
                    Less => self.binary_cmp(less_than)?,
                    Add => {
                        if self.peek(0).is_string() && self.peek(1).is_string() {
                            self.concatenate()?;
                        } else {
                            self.binary(std::ops::Add::add)?;
                        }
                    }
                    Subtract => self.binary(std::ops::Sub::sub)?,
                    Multiply => self.binary(std::ops::Mul::mul)?,
                    Divide => self.binary(std::ops::Div::div)?,
                    Remainder => self.binary(std::ops::Rem::rem)?,
                    Not => {
                        let v = self.pop()?;
                        self.push(Value::Boolean(!v.truthiness()));
                    }
                    Negate => {
                        if let Value::Number(_) = self.peek(0) {
                            let v = self.pop()?;
                            self.push((-f64::try_from(v)?).into());
                        } else {
                            let e = self.report_runtime_error("Operand must be a number.");
                            return Err(e);
                        }
                    }
                    Print => {
                        let v = self.pop()?;
                        println!("{}", v);
                    }
                    Return => {
                        //~ println!("{:?}", self.pop()?);
                        return Ok(());
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

    fn read_int(&mut self, size_in_bytes: usize) -> usize {
        let mut int = 0;
        for _ in 0..size_in_bytes {
            int = (int << 8) | self.read_byte() as usize;
        }
        int
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn peek(&self, depth: usize) -> &Value {
        &self.stack[self.stack.len() - depth - 1]
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, InterpretError> {
        self.stack
            .pop()
            .ok_or(InterpretError::Ice("Popped Empty Stack"))
    }

    fn binary(&mut self, f: fn(f64, f64) -> f64) -> Result<(), InterpretError> {
        if let Value::Number(_) = self.peek(0) {
            if let Value::Number(_) = self.peek(1) {
                let b = self.pop()?.try_into()?;
                let a = self.pop()?.try_into()?;
                self.push(f(a, b).into());
                return Ok(());
            }
        }

        let e = self.report_runtime_error("Both operands must be numbers");
        Err(e)
    }

    fn binary_cmp(&mut self, f: fn(f64, f64) -> bool) -> Result<(), InterpretError> {
        if let Value::Number(_) = self.peek(0) {
            if let Value::Number(_) = self.peek(1) {
                let b = self.pop()?.try_into()?;
                let a = self.pop()?.try_into()?;
                self.push(f(a, b).into());
                return Ok(());
            }
        }

        let e = self.report_runtime_error("Both operands must be numbers");
        Err(e)
    }

    fn report_runtime_error(&mut self, message: &'static str) -> InterpretError {
        let line = self.chunk.get_line(self.ip);
        eprintln!("[line {}] in script", line);
        self.reset_stack();
        InterpretError::RuntimeError(message)
    }

    fn concatenate(&mut self) -> Result<(), InterpretError> {
        let b = TryInto::<String>::try_into(self.pop()?)?;
        let a = TryInto::<String>::try_into(self.pop()?)?;

        let string = format!("{}{}", a, b);

        let v = self.add_object(string);

        self.push(v);

        Ok(())
    }

    fn add_object<T: Into<Object>>(&mut self, o: T) -> Value {
        let rc = Rc::new(o.into());
        self.objects.push(Rc::downgrade(&rc));
        Value::Object(rc)
    }

    /*
    fn trace(&mut self) -> { todo!() }

    fn resolve_cycles(&mut self) -> { todo!() }
    */
}

fn greater_than(a: f64, b: f64) -> bool {
    a > b
}
fn less_than(a: f64, b: f64) -> bool {
    a < b
}

#[derive(Debug)]
pub enum InterpretError {
    CompileError(&'static str),
    RuntimeError(&'static str),
    Ice(&'static str),
}

impl std::error::Error for InterpretError {}

impl std::fmt::Display for InterpretError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use InterpretError::*;
        match self {
            CompileError(s) => write!(f, "Compile Error: {}", s),
            RuntimeError(s) => write!(f, "Runtime Error: {}", s),
            Ice(s) => write!(f, "Internal Compiler Error: {}", s),
        }
    }
}
