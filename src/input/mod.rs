use crossterm::event::{Event, KeyEvent, KeyEventKind};
use crate::editor::EditorState;

pub mod debouncer;

pub async fn handle_event(event: Event, state: &mut EditorState) -> anyhow::Result<()> {
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
                        // TODO: Send to AI background thread
                        state.mode = crate::editor::Mode::Normal;
                        let query = state.ez_input.clone();
                        // For now, mock execution
                        execute_command(&format!("AI_RESULT_FOR: {}", query), state);
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
    // Phase 1: Just log it or apply a basic structural change
    // Since we're in insert mode and it debounced, we should delete the command characters from the buffer first!
    let chars_to_delete = cmd.len();
    if state.cursor_col >= chars_to_delete {
        state.cursor_col -= chars_to_delete;
        let line = state.buffer.line_to_char(state.cursor_row);
        state.buffer.remove(line + state.cursor_col..line + state.cursor_col + chars_to_delete);
    }
    
    // For now, let's just insert a dummy tag so we know it executed
    let execute_msg = format!("<EXEC: {}>", cmd);
    let line = state.buffer.line_to_char(state.cursor_row);
    state.buffer.insert(line + state.cursor_col, &execute_msg);
    state.cursor_col += execute_msg.len();
}
