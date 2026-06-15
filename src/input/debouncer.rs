use std::collections::HashMap;

#[derive(Default, Debug)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_command: bool,
}

impl TrieNode {
    fn insert(&mut self, word: &str) {
        let mut curr = self;
        for c in word.chars() {
            curr = curr.children.entry(c).or_default();
        }
        curr.is_command = true;
    }
}

pub struct Debouncer {
    buffer: Vec<char>,
    trie: TrieNode,
}

impl Debouncer {
    pub fn new() -> Self {
        let mut trie = TrieNode::default();
        // Load common vim commands into the trie
        let commands = vec!["ciw", "ci\"", "ci'", "cib", "ci(", "ci{", "ci[", "ci<",
                            "diw", "di\"", "di'", "dib", "di(", "di{", "di[", "di<",
                            "yiw", "yi\"", "yi'", "yib", "yi(", "yi{", "yi[", "yi<",
                            "d$", "c$", "y$", "dd", "cc", "yy", "daw", "daw", "yap"];
        for cmd in commands {
            trie.insert(cmd);
        }

        Self {
            buffer: Vec::with_capacity(10),
            trie,
        }
    }

    pub fn push(&mut self, c: char) {
        self.buffer.push(c);
        if self.buffer.len() > 10 {
            self.buffer.remove(0);
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Checks if the current buffer suffix matches a complete command
    pub fn matches_command(&self) -> Option<String> {
        // We check suffixes of the buffer.
        // e.g., if buffer is ['h', 'e', 'l', 'l', 'o', 'd', 'i', 'w'], suffix "diw" is a command.
        for i in 0..self.buffer.len() {
            let suffix: String = self.buffer[i..].iter().collect();
            if self.is_command(&suffix) {
                return Some(suffix);
            }
        }
        None
    }

    fn is_command(&self, word: &str) -> bool {
        let mut curr = &self.trie;
        for c in word.chars() {
            match curr.children.get(&c) {
                Some(node) => curr = node,
                None => return false,
            }
        }
        curr.is_command
    }
}
