use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
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
        }
    }
}
