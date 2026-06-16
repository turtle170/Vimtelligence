use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, BorderType},
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
    let highlights = state.syntax.get_highlights();
    
    // Calculate viewport scroll based on cursor position and available height
    let main_area = chunks[0];
    let height = main_area.height.saturating_sub(2); // Subtract borders
    let scroll_y = if state.cursor_row as u16 > height {
        (state.cursor_row as u16) - height + 2
    } else {
        0
    };

    let title = Line::from(vec![
        Span::styled(" Vim", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("telligence ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]);

    let mut current_byte = 0;
    let mut lines = Vec::new();
    let mut current_line_spans = Vec::new();

    let mut process_chunk = |start_byte: usize, end_byte: usize, color: Color| {
        if start_byte >= end_byte { return; }
        let chunk = &buffer_text[start_byte..end_byte];
        let mut parts = chunk.split('\n');
        
        if let Some(first) = parts.next() {
            if !first.is_empty() {
                current_line_spans.push(Span::styled(first.to_string(), Style::default().fg(color)));
            }
        }
        
        for part in parts {
            lines.push(Line::from(std::mem::take(&mut current_line_spans)));
            if !part.is_empty() {
                current_line_spans.push(Span::styled(part.to_string(), Style::default().fg(color)));
            }
        }
    };

    for (start, end, kind) in highlights {
        let start = start.min(buffer_text.len());
        let end = end.min(buffer_text.len());
        
        if start > current_byte {
            process_chunk(current_byte, start, Color::Reset);
        }
        if current_byte < end {
            let color = map_kind_to_ratatui_color(&kind);
            process_chunk(start.max(current_byte), end, color);
            current_byte = end;
        }
    }
    
    if current_byte < buffer_text.len() {
        process_chunk(current_byte, buffer_text.len(), Color::Reset);
    }
    lines.push(Line::from(current_line_spans));

    let main_buffer = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
        )
        .scroll((scroll_y, 0));
        
    f.render_widget(main_buffer, main_area);

    if state.mode != crate::editor::Mode::EzMode {
        let cursor_screen_y = (state.cursor_row as u16).saturating_sub(scroll_y) + 1;
        f.set_cursor((state.cursor_col + 1) as u16, cursor_screen_y);
    }

    let (keyshow_style, keyshow_text) = if state.mode == crate::editor::Mode::PendingMotion {
        (Style::default().fg(Color::Yellow), " [Pending] Waiting for motion or text object (e.g. w, p, \", {)")
    } else if state.mode == crate::editor::Mode::Insert {
        if state.debouncer.matches_command().is_some() {
            (Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD), " [Insert] Auto-switcher paused... (Executing in 1s or type to cancel)")
        } else {
            (Style::default().fg(Color::DarkGray), " [Insert] Type freely. Structural edits (like ciw) will be auto-detected.")
        }
    } else if state.mode == crate::editor::Mode::EzMode {
        (Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD), " [EZ MODE] Type your command in natural language and press Enter.")
    } else {
        (Style::default().fg(Color::DarkGray), "")
    };

    let keyshow = Paragraph::new(keyshow_text)
        .style(keyshow_style);
    f.render_widget(keyshow, chunks[1]);

    let (mode_color, mode_str) = match state.mode {
        crate::editor::Mode::Normal => (Color::Blue, " NORMAL "),
        crate::editor::Mode::Insert => (Color::Green, " INSERT "),
        crate::editor::Mode::Visual => (Color::Yellow, " VISUAL "),
        crate::editor::Mode::VisualLine => (Color::Yellow, " V-LINE "),
        crate::editor::Mode::PendingMotion => (Color::Cyan, " PENDING "),
        crate::editor::Mode::EzMode => (Color::Magenta, " EZ MODE "),
    };
    
    let pending_str = if state.pending_command.is_empty() {
        String::new()
    } else {
        format!(" {} ", state.pending_command)
    };
    
    let status_line = Line::from(vec![
        Span::styled(mode_str, Style::default().bg(mode_color).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(pending_str, Style::default().bg(Color::DarkGray).fg(Color::White)),
    ]);

    let status_paragraph = Paragraph::new(status_line).block(Block::default());
    f.render_widget(status_paragraph, chunks[2]);

    if state.mode == crate::editor::Mode::EzMode {
        let area = ratatui::layout::Rect::new(f.size().width / 4, f.size().height / 2 - 2, f.size().width / 2, 3);
        let overlay = Paragraph::new(state.ez_input.clone())
            .block(
                Block::default()
                    .title(" EZ Mode (Describe your edit) ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(Color::Magenta))
            );
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(overlay, area);
    }
}

fn map_kind_to_ratatui_color(kind: &str) -> Color {
    match kind {
        "keyword" | "return" | "if" | "else" | "let" | "fn" | "pub" | "struct" | "enum" | "match" => Color::Magenta,
        "string" | "string_literal" | "string_content" => Color::Green,
        "number" | "integer_literal" | "float_literal" | "boolean_literal" => Color::Yellow,
        "comment" | "line_comment" | "block_comment" => Color::DarkGray,
        "function" | "method" | "identifier" => Color::LightBlue,
        "type" | "type_identifier" | "primitive_type" => Color::Cyan,
        "punctuation" | "punctuation.delimiter" | "punctuation.bracket" | "{" | "}" | "(" | ")" | "[" | "]" | ";" | "," => Color::Gray,
        _ => Color::Reset,
    }
}
