use tree_sitter::{Language, Parser, Tree};
use tree_sitter_language_pack::get_language;
use ropey::Rope;

pub struct SyntaxEngine {
    parser: Parser,
    pub tree: Option<Tree>,
    language: Option<Language>,
}

impl SyntaxEngine {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            tree: None,
            language: None,
        }
    }

    pub fn set_language(&mut self, ext: &str) {
        // tree-sitter-language-pack uses extension strings to map languages
        let lang_opt = get_language(ext).ok();
        
        if let Some(l) = lang_opt {
            self.parser.set_language(&l).expect("Error loading language");
            self.language = Some(l);
        } else {
            self.language = None;
        }
    }

    pub fn parse(&mut self, text: &str) {
        if self.language.is_some() {
            self.tree = self.parser.parse(text, self.tree.as_ref());
        }
    }

    pub fn get_highlights(&self) -> Vec<(usize, usize, String)> {
        let mut highlights = Vec::new();
        if let Some(tree) = &self.tree {
            let mut cursor = tree.walk();
            let mut needs_next = true;
            
            while needs_next {
                let node = cursor.node();
                if node.child_count() == 0 {
                    // Leaf node
                    let kind = node.kind();
                    // Let's also include named nodes that might be useful
                    highlights.push((node.start_byte(), node.end_byte(), kind.to_string()));
                }
                
                if cursor.goto_first_child() {
                    continue;
                }
                
                while !cursor.goto_next_sibling() {
                    if !cursor.goto_parent() {
                        needs_next = false;
                        break;
                    }
                }
            }
        }
        // tree-sitter doesn't guarantee sorted highlights if we include parents, 
        // but since we only do leaf nodes, they are naturally sorted.
        highlights
    }

    pub fn get_node_at_byte(&self, byte_idx: usize) -> Option<(usize, usize)> {
        if let Some(tree) = &self.tree {
            let node = tree.root_node().descendant_for_byte_range(byte_idx, byte_idx)?;
            return Some((node.start_byte(), node.end_byte()));
        }
        None
    }

    pub fn get_enclosing_bounds(&self, byte_idx: usize, start_char: char, end_char: char, text: &str) -> Option<(usize, usize)> {
        if let Some(tree) = &self.tree {
            let mut node = tree.root_node().descendant_for_byte_range(byte_idx, byte_idx)?;
            
            loop {
                let start = node.start_byte();
                let end = node.end_byte();
                if start < end && end <= text.len() {
                    let s_char = text[start..].chars().next().unwrap_or('\0');
                    let e_char = text[..end].chars().next_back().unwrap_or('\0');
                    
                    if s_char == start_char && e_char == end_char {
                        return Some((start, end));
                    }
                }
                
                if let Some(parent) = node.parent() {
                    node = parent;
                } else {
                    break;
                }
            }
        }
        None
    }
}
