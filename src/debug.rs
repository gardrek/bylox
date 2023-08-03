/// A mapping from chunk byte offsets to file line numbers
/// Goal: fast insertion without regard for read speed
#[derive(Default)]
pub struct LineMap {
    pairs: Vec<(usize, usize)>,
}

impl LineMap {
    fn push(&mut self, offset: usize, line: usize) {
        self.pairs.push((offset, line));
    }

    pub fn add(&mut self, offset: usize, line: usize) {
        let len = self.pairs.len();

        if len == 0 {
            self.push(offset, line);
            return;
        }

        if line == self.pairs[len - 1].1 {
            return;
        }

        self.push(offset, line);
    }

    pub fn get_line(&self, offset: usize) -> usize {
        let mut last_line = 0;

        // we assume that there are no duplicate keys and that keys are in order
        for (next_offset, line) in self.pairs.iter() {
            if offset < *next_offset {
                return last_line;
            }
            last_line = *line;
        }

        last_line
    }
}
