use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Symbol(usize);

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default)]
pub struct Interner {
    strings: HashMap<String, Symbol>,
    next_string_id: usize,
}

impl Interner {
    pub fn intern_string(&mut self, string: String) -> Symbol {
        let id = Symbol(self.next_string_id);

        self.next_string_id += 1;

        self.strings.insert(string, id);

        id
    }
}
