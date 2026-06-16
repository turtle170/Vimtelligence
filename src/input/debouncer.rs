use crate::input::parser;

pub struct Debouncer {
    buffer: Vec<char>,
}

impl Debouncer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(20),
        }
    }

    pub fn push(&mut self, c: char) {
        self.buffer.push(c);
        if self.buffer.len() > 20 {
            self.buffer.remove(0);
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Checks if the current buffer suffix matches a complete command using AST parser
    pub fn matches_command(&self) -> Option<(String, parser::VimCommand)> {
        for i in 0..self.buffer.len() {
            let suffix: String = self.buffer[i..].iter().collect();
            
            if let parser::ParseResult::Complete(cmd) = parser::parse_command(&suffix) {
                // Heuristic: Ensure the character preceding the suffix is a boundary
                let is_boundary = if i == 0 {
                    true // Start of buffer
                } else {
                    let prev_char = self.buffer[i - 1];
                    prev_char.is_whitespace() || !prev_char.is_alphanumeric()
                };

                if is_boundary && cmd.operator.is_some() {
                    return Some((suffix, cmd));
                }
            }
        }
        None
    }
}
