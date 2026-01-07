use crossterm::style::Color;
use indoc::indoc;
use oelung_lantern::{render_multiple_test, spinner::bounce, BounceSpinner};

render_multiple_test! {
    name => bounce_spinner
    state => |sender| BounceSpinner::new(
        None,
        Some(Color::Red),
        sender
    )
    state_type => BounceSpinner
    send_and_receive => bounce::Tick
    expected_prefix => vec![
        indoc!(
            r#"
                <color={Red}>⠁</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⠂</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⠄</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⡀</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⢀</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⠠</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⠐</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⠈</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>⠁</>
            "#,
        ),
    ]
}
