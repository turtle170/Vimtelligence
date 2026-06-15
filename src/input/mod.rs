use crossterm::event::{Event, KeyEventKind, MouseEventKind, MouseButton};
use crate::editor::EditorState;

pub mod debouncer;
pub mod parser;

pub async fn handle_event(event: Event, state: &mut EditorState, ai_engine: &crate::ai::AiEngine) -> anyhow::Result<()> {
    match event {
        Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                if key.code == crossterm::event::KeyCode::Esc {
                    state.mode = crate::editor::Mode::Normal;
                    state.debouncer.clear();
                    state.pending_command.clear();
                    if let Some(_) = state.selection_anchor {
                        state.selection_anchor = None; // Exit visual mode
                    }
                    return Ok(());
                }

            match state.mode {
                crate::editor::Mode::Normal | crate::editor::Mode::Visual | crate::editor::Mode::VisualLine => {
                    // Quick commands
                    if state.pending_command.is_empty() {
                        if let crossterm::event::KeyCode::Char('i') = key.code {
                            state.mode = crate::editor::Mode::Insert;
                            return Ok(());
                        } else if let crossterm::event::KeyCode::Char('a') = key.code {
                            state.move_cursor_right();
                            state.mode = crate::editor::Mode::Insert;
                            return Ok(());
                        } else if let crossterm::event::KeyCode::Char('q') = key.code {
                            state.should_quit = true;
                            return Ok(());
                        } else if let crossterm::event::KeyCode::Char('s') = key.code {
                            let _ = state.save();
                            return Ok(());
                        } else if let crossterm::event::KeyCode::Char('W') = key.code {
                            if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                state.mode = crate::editor::Mode::EzMode;
                                state.ez_input.clear();
                                return Ok(());
                            }
                        }
                    }

                    if let crossterm::event::KeyCode::Char(c) = key.code {
                        state.pending_command.push(c);
                        
                        match parser::parse_command(&state.pending_command) {
                            parser::ParseResult::Complete(cmd) => {
                                execute_ast_command(&cmd, state);
                                state.pending_command.clear();
                            }
                            parser::ParseResult::Incomplete => {
                                // Wait for more characters
                            }
                            parser::ParseResult::Invalid => {
                                state.pending_command.clear();
                            }
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
        Event::Mouse(mouse) => {
            if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                // Determine scroll offset from ui/mod.rs logic
                // The main area is chunks[0], which starts at y=0.
                // Scroll is calculated as state.cursor_row - height + 2, but we don't know height here easily.
                // For simplicity, we just use the raw row and col, assuming no scroll for a toy demo.
                // To do this perfectly, we'd need to store scroll_y in EditorState.
                // Let's just set the row/col directly for now assuming no scrolling for short files.
                state.cursor_row = (mouse.row as usize).min(state.buffer.len_lines().saturating_sub(1));
                
                let line_len = state.buffer.line(state.cursor_row).len_chars();
                state.cursor_col = (mouse.column as usize).saturating_sub(1).min(line_len.saturating_sub(1));
                
                // If past the end of line, go to end of line
                if state.cursor_col > line_len {
                    state.cursor_col = line_len.saturating_sub(1);
                }
            }
        }
        _ => {}
    }
    Ok(())
}



pub fn execute_ast_command(cmd: &parser::VimCommand, state: &mut EditorState) {
    use parser::{Operator, Action, Motion, TextObject};

    // Special cases: Visual mode entry
    if let Some(Operator::Visual) = cmd.operator {
        state.mode = crate::editor::Mode::Visual;
        state.selection_anchor = Some((state.cursor_row, state.cursor_col));
        return;
    }
    if let Some(Operator::VisualLine) = cmd.operator {
        state.mode = crate::editor::Mode::VisualLine;
        state.selection_anchor = Some((state.cursor_row, 0));
        return;
    }

    // Determine the affected bounds (start_char, end_char)
    let mut bounds: Option<(usize, usize)> = None;

    match &cmd.action {
        Action::Motion(m) => {
            for _ in 0..cmd.count {
                match m {
                    Motion::Left => state.move_cursor_left(),
                    Motion::Right => state.move_cursor_right(),
                    Motion::Up => state.move_cursor_up(),
                    Motion::Down => state.move_cursor_down(),
                    Motion::StartOfLine => state.cursor_col = 0,
                    Motion::EndOfLine => {
                        let len = state.buffer.line(state.cursor_row).len_chars();
                        state.cursor_col = if len > 0 { len - 1 } else { 0 };
                    }
                    _ => {} // Other motions omitted for brevity, logic goes here
                }
            }
            if cmd.operator.is_some() {
                // If operator is applied with a motion, we need bounds.
                // For a proper implementation, we'd record the start position, apply the motion,
                // and the bounds would be between start and new position.
            }
        }
        Action::Inside(obj) | Action::Around(obj) => {
            // Re-use logic from `ciw` etc.
            let line = state.buffer.line(state.cursor_row).to_string();
            let col = state.cursor_col;
            let mut start = col;
            let mut end = col;

            match obj {
                TextObject::Word | TextObject::BigWord => {
                    while start > 0 && line.chars().nth(start - 1).map_or(false, |c| c.is_alphanumeric() || c == '_') {
                        start -= 1;
                    }
                    while end < line.len() && line.chars().nth(end).map_or(false, |c| c.is_alphanumeric() || c == '_') {
                        end += 1;
                    }
                    if matches!(cmd.action, Action::Around(_)) {
                        while end < line.len() && line.chars().nth(end).map_or(false, |c| c.is_whitespace()) {
                            end += 1;
                        }
                    }
                }
                _ => {} // Other text objects
            }
            
            if start < end {
                let char_idx = state.buffer.line_to_char(state.cursor_row);
                bounds = Some((char_idx + start, char_idx + end));
                state.cursor_col = start;
            }
        }
        Action::Line => {
            let start = state.buffer.line_to_char(state.cursor_row);
            let mut next_row = state.cursor_row + cmd.count;
            if next_row > state.buffer.len_lines() {
                next_row = state.buffer.len_lines();
            }
            let end = state.buffer.line_to_char(next_row);
            if start < end {
                bounds = Some((start, end));
                state.cursor_col = 0;
            }
        }
        Action::VisualSelection => {
            if let Some((anchor_row, anchor_col)) = state.selection_anchor {
                let start_idx = state.buffer.line_to_char(anchor_row) + anchor_col;
                let end_idx = state.buffer.line_to_char(state.cursor_row) + state.cursor_col;
                bounds = Some(if start_idx < end_idx { (start_idx, end_idx + 1) } else { (end_idx, start_idx + 1) });
            }
            state.mode = crate::editor::Mode::Normal;
            state.selection_anchor = None;
        }
    }

    if let Some(op) = &cmd.operator {
        if let Some((start, end)) = bounds {
            match op {
                Operator::Delete => {
                    state.clipboard_register = Some(state.buffer.slice(start..end).to_string());
                    state.buffer.remove(start..end);
                }
                Operator::Change => {
                    state.clipboard_register = Some(state.buffer.slice(start..end).to_string());
                    state.buffer.remove(start..end);
                    state.mode = crate::editor::Mode::Insert;
                }
                Operator::Yank => {
                    state.clipboard_register = Some(state.buffer.slice(start..end).to_string());
                }
                _ => {}
            }
        }
    }
}
