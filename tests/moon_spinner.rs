use indoc::indoc;
use oelung_lantern::{render_multiple_test, spinner::moon, MoonSpinner};

render_multiple_test! {
    name => moon_spinner
    state => |sender| MoonSpinner::new(
        None,
        sender
    )
    state_type => MoonSpinner
    send_and_receive => moon::Tick
    expected_prefix => vec![
        indoc!(
            r#"
                🌑
            "#,
        ),
        indoc!(
            r#"
                🌒
            "#,
        ),
        indoc!(
            r#"
                🌓
            "#,
        ),
        indoc!(
            r#"
                🌔
            "#,
        ),
        indoc!(
            r#"
                🌕
            "#,
        ),
        indoc!(
            r#"
                🌖
            "#,
        ),
        indoc!(
            r#"
                🌗
            "#,
        ),
        indoc!(
            r#"
                🌘
            "#,
        ),
        indoc!(
            r#"
                🌑
            "#,
        ),
    ]
}
