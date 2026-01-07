use indoc::indoc;
use oelung_lantern::{render_multiple_test, spinner::monkey, MonkeySpinner};

render_multiple_test! {
    name => monkey_spinner
    state => |sender| MonkeySpinner::new(
        None,
        sender
    )
    state_type => MonkeySpinner
    send_and_receive => monkey::Tick
    expected_prefix => vec![
        indoc!(
            r#"
                🙈
            "#,
        ),
        indoc!(
            r#"
                🙉
            "#,
        ),
        indoc!(
            r#"
                🙊
            "#,
        ),
        indoc!(
            r#"
                🐵
            "#,
        ),
        indoc!(
            r#"
                🙈
            "#,
        ),
    ]
}
