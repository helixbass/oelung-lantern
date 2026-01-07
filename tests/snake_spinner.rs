use oelung_lantern::{render_multiple_test, spinner::snake, SnakeSpinner};

render_multiple_test! {
    name => snake_spinner
    state => SnakeSpinner::new(
        None,
        Some(Color::Red),
        {SENDER}
    )
    receive => snake::Tick
    send => snake::Tick
}
