use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

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

pub fn is_simple_key_press(event: &Event, key: KeyCode) -> bool {
    is_key_press_with_modifiers(event, key, KeyModifiers::empty())
}

pub fn is_ctrl_char_press(event: &Event, ch: char) -> bool {
    is_key_press_with_modifiers(event, KeyCode::Char(ch), KeyModifiers::CONTROL)
}

fn is_key_press_with_modifiers(event: &Event, key: KeyCode, modifiers: KeyModifiers) -> bool {
    let Event::Key(event) = event else {
        return false;
    };
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
