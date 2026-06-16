use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, KeyEventKind, KeyEventState};
use egui::{Key, Modifiers};

pub fn translate_egui_key(key: Key, modifiers: Modifiers) -> Option<KeyEvent> {
    let code = match key {
        Key::ArrowDown => KeyCode::Down,
        Key::ArrowLeft => KeyCode::Left,
        Key::ArrowRight => KeyCode::Right,
        Key::ArrowUp => KeyCode::Up,
        Key::Escape => KeyCode::Esc,
        Key::Tab => KeyCode::Tab,
        Key::Backspace => KeyCode::Backspace,
        Key::Enter => KeyCode::Enter,
        Key::Space => KeyCode::Char(' '),
        Key::Insert => KeyCode::Insert,
        Key::Delete => KeyCode::Delete,
        Key::Home => KeyCode::Home,
        Key::End => KeyCode::End,
        Key::PageUp => KeyCode::PageUp,
        Key::PageDown => KeyCode::PageDown,
        Key::Num0 => KeyCode::Char('0'),
        Key::Num1 => KeyCode::Char('1'),
        Key::Num2 => KeyCode::Char('2'),
        Key::Num3 => KeyCode::Char('3'),
        Key::Num4 => KeyCode::Char('4'),
        Key::Num5 => KeyCode::Char('5'),
        Key::Num6 => KeyCode::Char('6'),
        Key::Num7 => KeyCode::Char('7'),
        Key::Num8 => KeyCode::Char('8'),
        Key::Num9 => KeyCode::Char('9'),
        Key::A => KeyCode::Char(if modifiers.shift { 'A' } else { 'a' }),
        Key::B => KeyCode::Char(if modifiers.shift { 'B' } else { 'b' }),
        Key::C => KeyCode::Char(if modifiers.shift { 'C' } else { 'c' }),
        Key::D => KeyCode::Char(if modifiers.shift { 'D' } else { 'd' }),
        Key::E => KeyCode::Char(if modifiers.shift { 'E' } else { 'e' }),
        Key::F => KeyCode::Char(if modifiers.shift { 'F' } else { 'f' }),
        Key::G => KeyCode::Char(if modifiers.shift { 'G' } else { 'g' }),
        Key::H => KeyCode::Char(if modifiers.shift { 'H' } else { 'h' }),
        Key::I => KeyCode::Char(if modifiers.shift { 'I' } else { 'i' }),
        Key::J => KeyCode::Char(if modifiers.shift { 'J' } else { 'j' }),
        Key::K => KeyCode::Char(if modifiers.shift { 'K' } else { 'k' }),
        Key::L => KeyCode::Char(if modifiers.shift { 'L' } else { 'l' }),
        Key::M => KeyCode::Char(if modifiers.shift { 'M' } else { 'm' }),
        Key::N => KeyCode::Char(if modifiers.shift { 'N' } else { 'n' }),
        Key::O => KeyCode::Char(if modifiers.shift { 'O' } else { 'o' }),
        Key::P => KeyCode::Char(if modifiers.shift { 'P' } else { 'p' }),
        Key::Q => KeyCode::Char(if modifiers.shift { 'Q' } else { 'q' }),
        Key::R => KeyCode::Char(if modifiers.shift { 'R' } else { 'r' }),
        Key::S => KeyCode::Char(if modifiers.shift { 'S' } else { 's' }),
        Key::T => KeyCode::Char(if modifiers.shift { 'T' } else { 't' }),
        Key::U => KeyCode::Char(if modifiers.shift { 'U' } else { 'u' }),
        Key::V => KeyCode::Char(if modifiers.shift { 'V' } else { 'v' }),
        Key::W => KeyCode::Char(if modifiers.shift { 'W' } else { 'w' }),
        Key::X => KeyCode::Char(if modifiers.shift { 'X' } else { 'x' }),
        Key::Y => KeyCode::Char(if modifiers.shift { 'Y' } else { 'y' }),
        Key::Z => KeyCode::Char(if modifiers.shift { 'Z' } else { 'z' }),
        Key::F1 => KeyCode::F(1),
        Key::F2 => KeyCode::F(2),
        Key::F3 => KeyCode::F(3),
        Key::F4 => KeyCode::F(4),
        Key::F5 => KeyCode::F(5),
        Key::F6 => KeyCode::F(6),
        Key::F7 => KeyCode::F(7),
        Key::F8 => KeyCode::F(8),
        Key::F9 => KeyCode::F(9),
        Key::F10 => KeyCode::F(10),
        Key::F11 => KeyCode::F(11),
        Key::F12 => KeyCode::F(12),
        _ => return None,
    };

    let mut mods = KeyModifiers::empty();
    if modifiers.shift { mods |= KeyModifiers::SHIFT; }
    if modifiers.ctrl { mods |= KeyModifiers::CONTROL; }
    if modifiers.alt { mods |= KeyModifiers::ALT; }
    if modifiers.command { mods |= KeyModifiers::META; }

    Some(KeyEvent {
        code,
        modifiers: mods,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    })
}
