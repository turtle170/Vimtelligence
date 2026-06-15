use crossterm::event::{Event, KeyEvent, KeyEventKind};
use crate::editor::EditorState;

pub mod debouncer;

pub async fn handle_event(event: Event, state: &mut EditorState, ai_engine: &crate::ai::AiEngine) -> anyhow::Result<()> {
    if let Event::Key(key) = event {
        if key.kind == KeyEventKind::Press {
            if key.code == crossterm::event::KeyCode::Esc {
                state.mode = crate::editor::Mode::Normal;
                state.debouncer.clear();
                return Ok(());
            }

            match state.mode {
                crate::editor::Mode::Normal => {
                    if let crossterm::event::KeyCode::Char('i') = key.code {
                        state.mode = crate::editor::Mode::Insert;
                    } else if let crossterm::event::KeyCode::Char('q') = key.code {
                        state.should_quit = true; // Temporary quit key for normal mode
                    } else if let crossterm::event::KeyCode::Char('s') = key.code {
                        let _ = state.save();
                    } else if let crossterm::event::KeyCode::Char('h') = key.code {
                        state.move_cursor_left();
                    } else if let crossterm::event::KeyCode::Char('l') = key.code {
                        state.move_cursor_right();
                    } else if let crossterm::event::KeyCode::Char('j') = key.code {
                        state.move_cursor_down();
                    } else if let crossterm::event::KeyCode::Char('k') = key.code {
                        state.move_cursor_up();
                    } else if let crossterm::event::KeyCode::Char('W') = key.code {
                        if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            state.mode = crate::editor::Mode::EzMode;
                            state.ez_input.clear();
                        }
                    }
                }
                crate::editor::Mode::Insert => {
                    if let crossterm::event::KeyCode::Char(c) = key.code {
                        state.debouncer.push(c);
                        
                        let col = state.cursor_col;
                        let line = state.buffer.line_to_char(state.cursor_row);
                        state.buffer.insert_char(line + col, c);
                        state.cursor_col += 1;
                    } else if let crossterm::event::KeyCode::Backspace = key.code {
                        if state.cursor_col > 0 {
                            state.cursor_col -= 1;
                            let line = state.buffer.line_to_char(state.cursor_row);
                            state.buffer.remove(line + state.cursor_col..line + state.cursor_col + 1);
                        }
                    }
                }
                crate::editor::Mode::EzMode => {
                    if let crossterm::event::KeyCode::Char(c) = key.code {
                        state.ez_input.push(c);
                    } else if let crossterm::event::KeyCode::Backspace = key.code {
                        state.ez_input.pop();
                    } else if let crossterm::event::KeyCode::Enter = key.code {
                        state.mode = crate::editor::Mode::Normal;
                        let query = state.ez_input.clone();
                        let _ = ai_engine.send_query(query).await;
                    }
                }
                _ => {}
            }
        } else if key.kind == KeyEventKind::Release {
            // Backspace aborts held operators, etc.
        }
    }
    Ok(())
}

pub fn execute_command(cmd: &str, state: &mut EditorState) {
    let chars_to_delete = if state.mode == crate::editor::Mode::Insert {
        cmd.len()
    } else {
        0
    };

    if state.cursor_col >= chars_to_delete {
        state.cursor_col -= chars_to_delete;
        let line = state.buffer.line_to_char(state.cursor_row);
        state.buffer.remove(line + state.cursor_col..line + state.cursor_col + chars_to_delete);
    }

    match cmd {
        "dd" => {
            let start = state.buffer.line_to_char(state.cursor_row);
            let next_row = state.cursor_row + 1;
            let end = if next_row < state.buffer.len_lines() {
                state.buffer.line_to_char(next_row)
            } else {
                state.buffer.len_chars()
            };
            if start < end {
                state.buffer.remove(start..end);
            }
            state.cursor_col = 0;
            if state.cursor_row >= state.buffer.len_lines() && state.cursor_row > 0 {
                state.cursor_row -= 1;
            }
        }
        "ciw" | "diw" | "daw" => {
            let line = state.buffer.line(state.cursor_row).to_string();
            let col = state.cursor_col;
            let mut start = col;
            while start > 0 && line.chars().nth(start - 1).map_or(false, |c| c.is_alphanumeric() || c == '_') {
                start -= 1;
            }
            let mut end = col;
            while end < line.len() && line.chars().nth(end).map_or(false, |c| c.is_alphanumeric() || c == '_') {
                end += 1;
            }
            
            if cmd == "daw" {
                while end < line.len() && line.chars().nth(end).map_or(false, |c| c.is_whitespace()) {
                    end += 1;
                }
            }

            if start < end {
                let char_idx = state.buffer.line_to_char(state.cursor_row);
                state.buffer.remove(char_idx + start..char_idx + end);
                state.cursor_col = start;
            }

            if cmd == "ciw" {
                state.mode = crate::editor::Mode::Insert;
            }
        }
        _ => {}
    }
}
