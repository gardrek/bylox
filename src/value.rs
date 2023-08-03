#[derive(Debug, Default, Clone, Copy)]
pub struct Value(pub f64);

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value(n)
    }
}

impl From<Value> for f64 {
    fn from(n: Value) -> Self {
        n.0
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
