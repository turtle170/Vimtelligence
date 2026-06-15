#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Change,
    Yank,
    VisualLine,
    Visual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    WordForward,
    WordBackward,
    WordEnd,
    BigWordForward,
    BigWordBackward,
    BigWordEnd,
    EndOfLine,
    StartOfLine,
    FirstNonBlank,
    Top,
    Bottom,
    FindChar(char),
    TillChar(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextObject {
    Word,
    BigWord,
    DoubleQuote,
    SingleQuote,
    Parentheses,
    Braces,
    Brackets,
    Paragraph,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Motion(Motion),
    Inside(TextObject),
    Around(TextObject),
    Line, // for dd, cc, yy
    VisualSelection, // Apply operator to current visual selection
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VimCommand {
    pub count: usize,
    pub operator: Option<Operator>,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseResult {
    Complete(VimCommand),
    Incomplete,
    Invalid,
}

pub fn parse_command(input: &str) -> ParseResult {
    if input.is_empty() {
        return ParseResult::Incomplete;
    }

    let mut chars = input.chars().peekable();
    let mut count_str = String::new();

    // Parse leading count
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() && (c != '0' || !count_str.is_empty()) {
            count_str.push(c);
            chars.next();
        } else {
            break;
        }
    }

    let count = if count_str.is_empty() { 1 } else { count_str.parse().unwrap_or(1) };

    if chars.peek().is_none() {
        return ParseResult::Incomplete; // Just a number
    }

    let next_char = *chars.peek().unwrap();
    let mut operator = None;
    let mut expect_motion = true;

    // Check operators
    match next_char {
        'd' => { operator = Some(Operator::Delete); chars.next(); }
        'c' => { operator = Some(Operator::Change); chars.next(); }
        'y' => { operator = Some(Operator::Yank); chars.next(); }
        'v' => {
            return ParseResult::Complete(VimCommand { count, operator: Some(Operator::Visual), action: Action::Motion(Motion::Right) }); // special case to enter visual
        }
        'V' => {
            return ParseResult::Complete(VimCommand { count, operator: Some(Operator::VisualLine), action: Action::Motion(Motion::Right) });
        }
        'D' => {
            return ParseResult::Complete(VimCommand { count, operator: Some(Operator::Delete), action: Action::Motion(Motion::EndOfLine) });
        }
        'C' => {
            return ParseResult::Complete(VimCommand { count, operator: Some(Operator::Change), action: Action::Motion(Motion::EndOfLine) });
        }
        _ => { expect_motion = false; }
    }

    if operator.is_some() {
        // If we have an operator, we expect a motion, text-object, or doubling (e.g. dd)
        if chars.peek().is_none() {
            return ParseResult::Incomplete;
        }
        let action_char = *chars.peek().unwrap();
        
        // Handle doubling
        if let Some(op) = &operator {
            if (op == &Operator::Delete && action_char == 'd') || 
               (op == &Operator::Change && action_char == 'c') ||
               (op == &Operator::Yank && action_char == 'y') {
                return ParseResult::Complete(VimCommand { count, operator, action: Action::Line });
            }
        }
    } else {
        // Standalone motion
        expect_motion = true;
    }

    if expect_motion {
        if chars.peek().is_none() {
            return ParseResult::Incomplete;
        }
        let m_char = chars.next().unwrap();
        
        match m_char {
            'h' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::Left) }),
            'l' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::Right) }),
            'j' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::Down) }),
            'k' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::Up) }),
            'w' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::WordForward) }),
            'W' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::BigWordForward) }),
            'b' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::WordBackward) }),
            'B' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::BigWordBackward) }),
            'e' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::WordEnd) }),
            'E' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::BigWordEnd) }),
            '$' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::EndOfLine) }),
            '0' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::StartOfLine) }),
            '^' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::FirstNonBlank) }),
            'g' => {
                if let Some(c) = chars.next() {
                    if c == 'g' {
                        return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::Top) });
                    }
                    return ParseResult::Invalid;
                }
                return ParseResult::Incomplete;
            }
            'G' => return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(Motion::Bottom) }),
            'f' | 'F' | 't' | 'T' => {
                if let Some(c) = chars.next() {
                    let motion = match m_char {
                        'f' => Motion::FindChar(c),
                        't' => Motion::TillChar(c),
                        // Note: F and T would be reverse directions, mapped to same for simplicity in this AST skeleton
                        _ => Motion::FindChar(c),
                    };
                    return ParseResult::Complete(VimCommand { count, operator, action: Action::Motion(motion) });
                }
                return ParseResult::Incomplete;
            }
            'i' | 'a' => {
                if operator.is_none() {
                    // i and a without operator are enter insert mode
                    return ParseResult::Invalid; // Handle directly in input loop, not as a motion
                }
                if let Some(c) = chars.next() {
                    let obj = match c {
                        'w' => TextObject::Word,
                        'W' => TextObject::BigWord,
                        '"' => TextObject::DoubleQuote,
                        '\'' => TextObject::SingleQuote,
                        '(' | ')' => TextObject::Parentheses,
                        '{' | '}' => TextObject::Braces,
                        '[' | ']' => TextObject::Brackets,
                        'p' => TextObject::Paragraph,
                        _ => return ParseResult::Invalid,
                    };
                    if m_char == 'i' {
                        return ParseResult::Complete(VimCommand { count, operator, action: Action::Inside(obj) });
                    } else {
                        return ParseResult::Complete(VimCommand { count, operator, action: Action::Around(obj) });
                    }
                }
                return ParseResult::Incomplete;
            }
            _ => return ParseResult::Invalid,
        }
    }

    ParseResult::Invalid
}
