use crossterm::{
    event::{self, EventStream, PushKeyboardEnhancementFlags},
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
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        PushKeyboardEnhancementFlags(
            crossterm::event::KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | crossterm::event::KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        )
    )?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = EditorState::new();
    let mut event_stream = EventStream::new();

    // Main loop
    loop {
        terminal.draw(|f| ui::render(f, &state))?;

        if state.should_quit {
            break;
        }

        // If in insert mode and we have a potential command match, wait with timeout
        let debounce_timeout = std::time::Duration::from_millis(150);
        let has_match = state.mode == editor::Mode::Insert && state.debouncer.matches_command().is_some();

        if has_match {
            match tokio::time::timeout(debounce_timeout, event_stream.next()).await {
                Ok(Some(Ok(event))) => {
                    // A key was pressed within the 150ms window.
                    // Pass to handler (if it's an alphabetic key, handler should clear debouncer)
                    input::handle_event(event, &mut state).await?;
                }
                Ok(Some(Err(e))) => return Err(e.into()),
                Ok(None) => break,
                Err(_) => {
                    // Timeout hit! Execute the command
                    let cmd = state.debouncer.matches_command().unwrap();
                    input::execute_command(&cmd, &mut state);
                    state.debouncer.clear();
                    // Keep user in Insert mode seamlessly
                }
            }
        } else {
            // Normal blocking wait
            if let Some(Ok(event)) = event_stream.next().await {
                input::handle_event(event, &mut state).await?;
            }
        }
    }

    // Restore terminal
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::PopKeyboardEnhancementFlags
    )?;
    disable_raw_mode()?;
    Ok(())
}
