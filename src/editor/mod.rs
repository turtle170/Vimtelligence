use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    VisualLine,
    PendingMotion,
    EzMode,
}

pub struct EditorState {
    pub buffer: Rope,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub mode: Mode,
    pub should_quit: bool,
    pub debouncer: crate::input::debouncer::Debouncer,
    pub ez_input: String,
    pub file_path: Option<String>,
    pub pending_command: String,
    pub selection_anchor: Option<(usize, usize)>, // (row, col)
    pub clipboard_register: Option<String>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            buffer: Rope::from_str(""),
            cursor_row: 0,
            cursor_col: 0,
            mode: Mode::Normal,
            should_quit: false,
            debouncer: crate::input::debouncer::Debouncer::new(),
            ez_input: String::new(),
            file_path: None,
            pending_command: String::new(),
            selection_anchor: None,
            clipboard_register: None,
        }
    }

    pub fn from_path(path: &str) -> anyhow::Result<Self> {
        let buffer = if std::path::Path::new(path).exists() {
            let text = std::fs::read_to_string(path)?;
            Rope::from_str(&text)
        } else {
            Rope::from_str("")
        };

        Ok(Self {
            buffer,
            cursor_row: 0,
            cursor_col: 0,
            mode: Mode::Normal,
            should_quit: false,
            debouncer: crate::input::debouncer::Debouncer::new(),
            ez_input: String::new(),
            file_path: Some(path.to_string()),
            pending_command: String::new(),
            selection_anchor: None,
            clipboard_register: None,
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = &self.file_path {
            let file = std::fs::File::create(path)?;
            self.buffer.write_to(file)?;
        }
        Ok(())
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.buffer.line(self.cursor_row).len_chars();
        // Allow moving to the character just after the last one (for appending)
        // In Normal mode it usually stops at line_len - 1, but for simplicity we use line_len
        let max_col = if line_len > 0 { line_len - 1 } else { 0 };
        // Wait, if line ends with newline, we shouldn't move onto the newline.
        // Let's just use a simple clamp.
        if self.cursor_col < max_col {
            self.cursor_col += 1;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let line_len = self.buffer.line(self.cursor_row).len_chars();
            let max_col = if line_len > 0 { line_len - 1 } else { 0 };
            if self.cursor_col > max_col {
                self.cursor_col = max_col;
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        let max_row = if self.buffer.len_lines() > 0 { self.buffer.len_lines() - 1 } else { 0 };
        if self.cursor_row < max_row {
            self.cursor_row += 1;
            let line_len = self.buffer.line(self.cursor_row).len_chars();
            let max_col = if line_len > 0 { line_len - 1 } else { 0 };
            if self.cursor_col > max_col {
                self.cursor_col = max_col;
            }
        }
    }
}
