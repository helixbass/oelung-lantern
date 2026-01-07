use crossterm::style::Color;
use indoc::indoc;
use oelung_lantern::{render_multiple_test, spinner::phase, PhaseSpinner};

render_multiple_test! {
    name => phase_spinner
    state => |sender| PhaseSpinner::new(
        None,
        Some(Color::Red),
        sender
    )
    state_type => PhaseSpinner
    send_and_receive => phase::Tick
    expected_prefix => vec![
        indoc!(
            r#"
                <color={Red}>█</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▉</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▊</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▋</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▌</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▍</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▎</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▏</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▎</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▍</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▌</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▋</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▊</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>▉</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>█</>
            "#,
        ),
    ]
}
