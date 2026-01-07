use crossterm::style::Color;
use oelung_lantern::{render_multiple_test, spinner::snake, SnakeSpinner};

render_multiple_test! {
    name => snake_spinner
    state => |sender| SnakeSpinner::new(
        None,
        Some(Color::Red),
        sender
    )
    state_type => SnakeSpinner
    send_and_receive => snake::Tick
}
