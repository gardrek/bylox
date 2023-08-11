use crate::vm::InterpretError;
use std::rc::Rc;

#[derive(Clone)]
pub enum Value {
    //~ #[default]
    Nil,
    Boolean(bool),
    Number(f64),
    Object(Rc<Object>),
}

pub struct Object {
    pub kind: ObjectKind,
}

pub enum ObjectKind {
    String(String),
}

impl Value {
    pub fn truthiness(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::Object(p) => match &p.kind {
                ObjectKind::String(_) => true,
                _ => false,
            },
            _ => false,
        }
    }
}

/*
#[derive(Default)]
pub struct TraceMap {
    pub vec: Vec<(Rc<Object>, usize)>,
}

impl TraceMap {
    /// adds one to the count for a node, or inserts it if it's not existent
    /// returns the count of the node
    fn add(&mut self, key: Rc<Object>) -> usize {
        let index = 'index: {
            for (i, (k, _)) in self.vec.iter().enumerate() {
                if Rc::ptr_eq(&key, k) {
                    break 'index Some(i);
                }
            }
            None
        };

        if let Some(i) = index {
            self.vec[i].1 += 1;
            self.vec[i].1
        } else {
            self.vec.push((key, 1));
            1
        }
    }

    fn get_count(&mut self, key: Rc<Object>) -> usize {
        let index = 'index: {
            for (i, (k, _)) in self.vec.iter().enumerate() {
                if Rc::ptr_eq(&key, k) {
                    break 'index Some(i);
                }
            }
            None
        };

        if let Some(i) = index {
            self.vec[i].1
        } else {
            0
        }
    }

    pub fn trace(&mut self, value: &Value) {
        match value {
            Value::Object(obj) => {
                if self.add(obj.clone()) == 1 {
                    match &obj.kind {
                        ObjectKind::String(_) => (),
                    }
                }
            }
            _ => (),
        }
    }
}
*/

impl TryFrom<Value> for f64 {
    type Error = InterpretError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok(n),
            _ => Err(InterpretError::Ice("Not a number")),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = InterpretError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(p) => match &p.kind {
                ObjectKind::String(s) => Ok(s.to_string()),
                _ => Err(InterpretError::Ice("Not a string")),
            },
            _ => Err(InterpretError::Ice("Not a string")),
        }
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<bool> for Value {
    fn from(n: bool) -> Self {
        Value::Boolean(n)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Object(
            Object {
                kind: ObjectKind::String(s),
            }
            .into(),
        )
    }
}

impl From<String> for Object {
    fn from(s: String) -> Self {
        Object {
            kind: ObjectKind::String(s),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::Object(p) => match &p.kind {
                ObjectKind::String(s) => write!(f, "{}", s),
                //~ _ => write!(f, "Object#{:p}", p),
            },
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Nil | Value::Boolean(_) | Value::Number(_) => write!(f, "#{}", self),
            Value::Object(p) => match &p.kind {
                ObjectKind::String(s) => write!(f, "String#\"{}\"", s.escape_debug()),
                //~ _ => write!(f, "Object#{:p}", p),
            },
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Boolean(a), Boolean(b)) => a == b,
            (Number(a), Number(b)) => a == b,
            (Object(a), Object(b)) => match (&a.kind, &b.kind) {
                (ObjectKind::String(s_a), ObjectKind::String(s_b)) => s_a == s_b,
                _ => Rc::ptr_eq(a, b),
            },
            (Nil, _) | (Boolean(_), _) | (Number(_), _) | (Object(_), _) => false,
        }
    }
}
