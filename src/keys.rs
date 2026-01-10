use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub fn is_simple_char_press(event: &Event, ch: char) -> bool {
    is_simple_key_press(event, KeyCode::Char(ch))
        || matches!(
            event,
            Event::Key(key_event) if matches!(
                key_event.code,
                KeyCode::Char(ch) if ch >= 'A' && ch <= 'Z'
            )
        ) && is_key_press_with_modifiers(event, KeyCode::Char(ch), KeyModifiers::SHIFT)
}

pub fn is_simple_key_press_key_event(event: &KeyEvent, key: KeyCode) -> bool {
    is_key_press_with_modifiers_key_event(event, key, KeyModifiers::empty())
}

pub fn is_simple_key_press(event: &Event, key: KeyCode) -> bool {
    is_key_press_with_modifiers(event, key, KeyModifiers::empty())
}

pub fn is_ctrl_char_press(event: &Event, ch: char) -> bool {
    is_key_press_with_modifiers(event, KeyCode::Char(ch), KeyModifiers::CONTROL)
}

fn is_key_press_with_modifiers_key_event(
    event: &KeyEvent,
    key: KeyCode,
    modifiers: KeyModifiers,
) -> bool {
    if event.code != key {
        return false;
    }
    if event.modifiers != modifiers {
        return false;
    }
    if event.kind != KeyEventKind::Press {
        return false;
    }
    true
}

fn is_key_press_with_modifiers(event: &Event, key: KeyCode, modifiers: KeyModifiers) -> bool {
    let Event::Key(event) = event else {
        return false;
    };
    is_key_press_with_modifiers_key_event(event, key, modifiers)
}

pub fn is_any_simple_char_press_key_event(event: &KeyEvent) -> Option<char> {
    let KeyCode::Char(ch) = event.code else {
        return None;
    };
    match event.modifiers {
        KeyModifiers::NONE => {}
        KeyModifiers::SHIFT => {
            if !(ch >= 'A' && ch <= 'Z') {
                return None;
            }
        }
        _ => return None,
    }
    Some(ch)
}

pub fn is_any_simple_char_press(event: &Event) -> Option<char> {
    let Event::Key(event) = event else {
        return None;
    };
    is_any_simple_char_press_key_event(event)
}

pub fn is_simple_digit_press(event: &Event) -> Option<char> {
    is_any_simple_char_press(event).filter(|ch| *ch >= '0' && *ch <= '9')
}
