use crossterm::style::Color;
use indoc::indoc;
use oelung_lantern::{render_multiple_test, spinner::bar, BarSpinner};

render_multiple_test! {
    name => bar_spinner
    state => |sender| BarSpinner::new(
        None,
        Some(Color::Red),
        sender
    )
    state_type => BarSpinner
    send_and_receive => bar::Tick
    expected_prefix => vec![
        indoc!(
            r#"
                <color={Red}>|</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>/</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>-</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>\</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>|</>
            "#,
        ),
    ]
}
