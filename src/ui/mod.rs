use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::editor::EditorState;

pub fn render(f: &mut Frame, state: &EditorState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1), // Main Buffer
            Constraint::Length(1), // Keyshow Dynamic Hint Bar
            Constraint::Length(1), // Status Line
        ])
        .split(f.size());

    let buffer_text = String::from(state.buffer.clone());
    let main_buffer = Paragraph::new(buffer_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(main_buffer, chunks[0]);

    let keyshow_text = if state.mode == crate::editor::Mode::PendingMotion {
        "Keyshow: [c]hange -> [i]nside -> [w]ord, [p]aragraph, [\"]quotes"
    } else if state.mode == crate::editor::Mode::Insert {
        if state.debouncer.matches_command().is_some() {
            "Keyshow: Auto-switcher paused... (Type another letter to cancel, or wait to execute)"
        } else {
            "Keyshow: Type freely. Vim commands will be auto-detected."
        }
    } else if state.mode == crate::editor::Mode::EzMode {
        "Keyshow: EZ MODE active. Type your command in natural language and press Enter."
    } else {
        "Keyshow Bar (Inactive)"
    };

    let keyshow = Paragraph::new(keyshow_text).block(Block::default().borders(Borders::TOP));
    f.render_widget(keyshow, chunks[1]);

    let mode_str = match state.mode {
        crate::editor::Mode::Normal => "NORMAL",
        crate::editor::Mode::Insert => "INSERT",
        crate::editor::Mode::PendingMotion => "PENDING",
        crate::editor::Mode::EzMode => "EZ MODE",
    };
    let status_line = Paragraph::new(format!("-- {} --", mode_str)).block(Block::default());
    f.render_widget(status_line, chunks[2]);

    if state.mode == crate::editor::Mode::EzMode {
        let area = ratatui::layout::Rect::new(f.size().width / 4, f.size().height / 2 - 2, f.size().width / 2, 3);
        let overlay = Paragraph::new(state.ez_input.clone())
            .block(Block::default().title("EZ Mode (Describe your edit)").borders(Borders::ALL));
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(overlay, area);
    }
}
