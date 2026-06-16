use eframe::egui;
use vimtelligence::editor::{EditorState, Mode};
use vimtelligence::ai::AiEngine;
use vimtelligence::input;
use crossterm::event::Event;
use std::env;
use std::time::{Instant, Duration};
use tokio::sync::mpsc;
use tokio::runtime::Runtime;

struct VimtelligenceApp {
    state: EditorState,
    ai_engine: AiEngine,
    runtime: Runtime,
    last_input_time: Instant,
}

impl VimtelligenceApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        cc.egui_ctx.set_fonts(fonts);
        
        let args: Vec<String> = env::args().collect();
        let file_path = if args.len() > 1 {
            Some(args[1].clone())
        } else {
            None
        };

        let mut state = EditorState::new();
        state.file_path = file_path.clone();
        if let Some(path) = &state.file_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                state.buffer = ropey::Rope::from_str(&content);
            }
        }

        let runtime = Runtime::new().unwrap();
        // Since we removed model path initialization temporarily
        let ai_engine = AiEngine::new(std::path::PathBuf::new());

        Self {
            state,
            ai_engine,
            runtime,
            last_input_time: Instant::now(),
        }
    }
}

impl eframe::App for VimtelligenceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Process AI responses non-blocking
        while let Ok(msg) = self.ai_engine.rx.try_recv() {
            match msg {
                vimtelligence::ai::AiResponse::Command(cmd) => {
                    self.state.mode = Mode::Normal;
                    if let vimtelligence::input::parser::ParseResult::Complete(ast) = vimtelligence::input::parser::parse_command(&cmd) {
                        vimtelligence::input::execute_ast_command(&ast, &mut self.state);
                    }
                }
                vimtelligence::ai::AiResponse::Error(_) => {}
            }
        }

        if self.state.should_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // 2. Process Input
        let mut events = vec![];
        ctx.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Key { key, pressed, modifiers, .. } => {
                        if *pressed {
                            if let Some(crossterm_key) = input::egui_compat::translate_egui_key(*key, *modifiers) {
                                events.push(Event::Key(crossterm_key));
                            }
                        }
                    }
                    egui::Event::Text(t) => {
                        for c in t.chars() {
                            if !c.is_control() {
                                events.push(Event::Key(crossterm::event::KeyEvent {
                                    code: crossterm::event::KeyCode::Char(c),
                                    modifiers: crossterm::event::KeyModifiers::empty(),
                                    kind: crossterm::event::KeyEventKind::Press,
                                    state: crossterm::event::KeyEventState::empty(),
                                }));
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        if !events.is_empty() {
            self.last_input_time = Instant::now();
        }

        for ev in events {
            self.runtime.block_on(async {
                let _ = input::handle_event(ev, &mut self.state, &self.ai_engine).await;
            });
        }

        // 3. Handle debouncer timeout (1000ms delay)
        let has_match = self.state.mode == Mode::Insert && self.state.debouncer.matches_command().is_some();
        if has_match {
            if self.last_input_time.elapsed() >= Duration::from_millis(1000) {
                let (cmd_str, cmd_ast) = self.state.debouncer.matches_command().unwrap();
                let len = cmd_str.len();
                if self.state.cursor_col >= len {
                    self.state.cursor_col -= len;
                    let line = self.state.buffer.line_to_char(self.state.cursor_row);
                    self.state.buffer.remove(line + self.state.cursor_col..line + self.state.cursor_col + len);
                }
                input::execute_ast_command(&cmd_ast, &mut self.state);
                self.state.debouncer.clear();
            } else {
                ctx.request_repaint_after(Duration::from_millis(50));
            }
        }

        // 4. Render UI
        let status_frame = egui::Frame::default()
            .fill(if ctx.style().visuals.dark_mode { egui::Color32::from_rgb(10, 25, 30) } else { egui::Color32::from_rgb(220, 240, 250) })
            .inner_margin(4.0);

        egui::TopBottomPanel::bottom("status_bar").frame(status_frame).show(ctx, |ui| {
            let (bg_color, fg_color, mode_str) = match self.state.mode {
                Mode::Normal => (
                    if ui.visuals().dark_mode { egui::Color32::from_gray(60) } else { egui::Color32::from_gray(200) },
                    if ui.visuals().dark_mode { egui::Color32::WHITE } else { egui::Color32::BLACK },
                    " NORMAL "
                ),
                Mode::Insert => (egui::Color32::from_rgb(40, 200, 40), egui::Color32::BLACK, " INSERT "),
                Mode::Visual | Mode::VisualLine => (egui::Color32::from_rgb(0, 0, 139), egui::Color32::WHITE, " VISUAL "),
                Mode::PendingMotion => (egui::Color32::from_rgb(200, 200, 40), egui::Color32::BLACK, " PENDING "),
                Mode::EzMode => (egui::Color32::from_rgb(0, 255, 255), egui::Color32::BLACK, " EZ MODE "),
            };

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(mode_str)
                        .background_color(bg_color)
                        .color(fg_color)
                        .strong()
                );
                
                if !self.state.pending_command.is_empty() {
                    ui.label(format!(" {} ", self.state.pending_command));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(path) = &self.state.file_path {
                        ui.label(path);
                    } else {
                        ui.label("[No Name]");
                    }
                });
            });
            
            if self.state.mode == Mode::EzMode {
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("EZ MODE: {}", self.state.ez_input))
                        .color(egui::Color32::from_rgb(0, 255, 255))
                        .strong()
                        .size(16.0)
                );
            } else if self.state.mode == Mode::Insert && has_match {
                let remaining = 1000i32 - self.last_input_time.elapsed().as_millis() as i32;
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("Executing structural command in {}ms... (Type to cancel)", remaining.max(0)))
                        .color(egui::Color32::RED)
                        .strong()
                );
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let text = String::from(self.state.buffer.clone());
                let mut output = egui::text::LayoutJob::default();
                
                let cursor_char_idx = self.state.buffer.line_to_char(self.state.cursor_row) + self.state.cursor_col;
                let cursor_byte_idx = self.state.buffer.char_to_byte(cursor_char_idx);
                
                let highlights = self.state.syntax.get_highlights();
                let mut current_byte = 0;
                let dark_mode = ui.visuals().dark_mode;
                let default_color = ui.visuals().text_color();
                let bg_color = ui.visuals().window_fill();
                
                let mut append_text = |out: &mut egui::text::LayoutJob, chunk: &str, color: egui::Color32, is_cursor: bool| {
                    let mut text_color = color;
                    let mut background = egui::Color32::TRANSPARENT;
                    if is_cursor {
                        text_color = bg_color;
                        background = default_color;
                    }
                    if text_color == egui::Color32::TRANSPARENT {
                        text_color = default_color;
                    }
                    
                    out.append(chunk, 0.0, egui::TextFormat {
                        font_id: egui::FontId::monospace(14.0),
                        color: text_color,
                        background,
                        ..Default::default()
                    });
                };

                let mut cursor_rendered = false;
                
                let mut process_chunk = |out: &mut egui::text::LayoutJob, start_byte: usize, end_byte: usize, color: egui::Color32| {
                    if start_byte >= end_byte { return; }
                    
                    if !cursor_rendered && cursor_byte_idx >= start_byte && cursor_byte_idx < end_byte {
                        let cursor_char_len = text[cursor_byte_idx..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                        let cursor_end_byte = cursor_byte_idx + cursor_char_len;
                        
                        if start_byte < cursor_byte_idx {
                            append_text(out, &text[start_byte..cursor_byte_idx], color, false);
                        }
                        
                        let end_c = cursor_end_byte.min(end_byte);
                        append_text(out, &text[cursor_byte_idx..end_c], color, true);
                        cursor_rendered = true;
                        
                        if end_c < end_byte {
                            append_text(out, &text[end_c..end_byte], color, false);
                        }
                    } else {
                        append_text(out, &text[start_byte..end_byte], color, false);
                    }
                };
                
                for (start, end, kind) in highlights {
                    let start = start.min(text.len());
                    let end = end.min(text.len());
                    
                    if start > current_byte {
                        process_chunk(&mut output, current_byte, start, egui::Color32::TRANSPARENT);
                    }
                    if current_byte < end {
                        let color = map_kind_to_color(&kind, dark_mode);
                        process_chunk(&mut output, start.max(current_byte), end, color);
                        current_byte = end;
                    }
                }
                
                if current_byte < text.len() {
                    process_chunk(&mut output, current_byte, text.len(), egui::Color32::TRANSPARENT);
                }
                
                if !cursor_rendered {
                    append_text(&mut output, " ", egui::Color32::TRANSPARENT, true);
                }
                
                ui.label(output);
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Vimtelligence"),
        ..Default::default()
    };
    eframe::run_native(
        "Vimtelligence",
        options,
        Box::new(|cc| Box::new(VimtelligenceApp::new(cc)) as Box<dyn eframe::App>),
    )
}

fn map_kind_to_color(kind: &str, dark_mode: bool) -> egui::Color32 {
    let (d, l) = match kind {
        "keyword" | "return" | "if" | "else" | "let" | "fn" | "pub" | "struct" | "enum" | "match" => (egui::Color32::from_rgb(198, 120, 221), egui::Color32::from_rgb(166, 38, 164)),
        "string" | "string_literal" | "string_content" => (egui::Color32::from_rgb(152, 195, 121), egui::Color32::from_rgb(80, 161, 79)),
        "number" | "integer_literal" | "float_literal" | "boolean_literal" => (egui::Color32::from_rgb(209, 154, 102), egui::Color32::from_rgb(152, 104, 1)),
        "comment" | "line_comment" | "block_comment" => (egui::Color32::from_rgb(92, 99, 112), egui::Color32::from_rgb(160, 161, 167)),
        "function" | "method" | "identifier" => (egui::Color32::from_rgb(97, 175, 239), egui::Color32::from_rgb(64, 120, 242)),
        "type" | "type_identifier" | "primitive_type" => (egui::Color32::from_rgb(229, 192, 123), egui::Color32::from_rgb(193, 132, 1)),
        "punctuation" | "punctuation.delimiter" | "punctuation.bracket" | "{" | "}" | "(" | ")" | "[" | "]" | ";" | "," => (egui::Color32::from_rgb(171, 178, 191), egui::Color32::from_rgb(56, 58, 66)),
        _ => (egui::Color32::TRANSPARENT, egui::Color32::TRANSPARENT),
    };
    if dark_mode { d } else { l }
}
