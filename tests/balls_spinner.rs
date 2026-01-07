use crossterm::style::Color;
use indoc::indoc;
use oelung_lantern::{render_multiple_test, spinner::balls, BallsSpinner};

render_multiple_test! {
    name => balls_spinner
    state => |sender| BallsSpinner::new(
        None,
        Some(Color::Red),
        sender
    )
    state_type => BallsSpinner
    send_and_receive => balls::Tick
    expected_prefix => vec![
        indoc!(
            r#"
                <color={Red}>⢄</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⢂</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⢁</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⡁</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⡈</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⡐</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⡠</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⢄</>
            "#,
        ),
    ]
}
