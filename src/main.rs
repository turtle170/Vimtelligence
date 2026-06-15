use crossterm::{
    event::{EnableMouseCapture, DisableMouseCapture, EventStream, PushKeyboardEnhancementFlags},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use futures::StreamExt;

mod editor;
mod input;
mod ui;
mod ai;

use editor::EditorState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut state = if args.len() > 1 {
        EditorState::from_path(&args[1])?
    } else {
        EditorState::new()
    };

    let model_path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".vimtelligence/models/gemma-3-270m-it-UD-Q8_K_XL.gguf");
    let mut ai_engine = ai::AiEngine::new(model_path);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        PushKeyboardEnhancementFlags(
            crossterm::event::KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | crossterm::event::KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        )
    )?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut event_stream = EventStream::new();

    // Main loop
    loop {
        terminal.draw(|f| ui::render(f, &state))?;

        if state.should_quit {
            break;
        }

        let debounce_timeout = std::time::Duration::from_millis(150);
        let has_match = state.mode == editor::Mode::Insert && state.debouncer.matches_command().is_some();

        if has_match {
            match tokio::time::timeout(debounce_timeout, event_stream.next()).await {
                Ok(Some(Ok(event))) => {
                    let mut retro_execute = false;
                    if let crossterm::event::Event::Key(k) = &event {
                        if k.kind == crossterm::event::KeyEventKind::Press && 
                           (k.code == crossterm::event::KeyCode::Char(' ') || k.code == crossterm::event::KeyCode::Enter) {
                            retro_execute = true;
                        }
                    }

                    if retro_execute {
                        let (cmd_str, cmd_ast) = state.debouncer.matches_command().unwrap();
                        let len = cmd_str.len();
                        if state.cursor_col >= len {
                            state.cursor_col -= len;
                            let line = state.buffer.line_to_char(state.cursor_row);
                            state.buffer.remove(line + state.cursor_col..line + state.cursor_col + len);
                        }
                        input::execute_ast_command(&cmd_ast, &mut state);
                        state.debouncer.clear();
                    }
                    input::handle_event(event, &mut state, &ai_engine).await?;
                }
                Ok(Some(Err(e))) => return Err(e.into()),
                Ok(None) => break,
                Err(_) => {
                    let (cmd_str, cmd_ast) = state.debouncer.matches_command().unwrap();
                    let len = cmd_str.len();
                    if state.cursor_col >= len {
                        state.cursor_col -= len;
                        let line = state.buffer.line_to_char(state.cursor_row);
                        state.buffer.remove(line + state.cursor_col..line + state.cursor_col + len);
                    }
                    input::execute_ast_command(&cmd_ast, &mut state);
                    state.debouncer.clear();
                }
            }
        } else {
            tokio::select! {
                maybe_event = event_stream.next() => {
                    if let Some(Ok(event)) = maybe_event {
                        input::handle_event(event, &mut state, &ai_engine).await?;
                    }
                }
                Some(response) = ai_engine.rx.recv() => {
                    match response {
                        ai::AiResponse::Command(cmd) => {
                            if let input::parser::ParseResult::Complete(ast) = input::parser::parse_command(&cmd) {
                                input::execute_ast_command(&ast, &mut state);
                            }
                        }
                        ai::AiResponse::Error(_) => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        crossterm::event::PopKeyboardEnhancementFlags
    )?;
    disable_raw_mode()?;
    Ok(())
}
