use indoc::indoc;
use oelung_lantern::{render_multiple_test, spinner::world, WorldSpinner};

render_multiple_test! {
    name => world_spinner
    state => |sender| WorldSpinner::new(
        None,
        sender
    )
    state_type => WorldSpinner
    send_and_receive => world::Tick
    expected_prefix => vec![
        indoc!(
            r#"
                🌍
            "#,
        ),
        indoc!(
            r#"
                🌎
            "#,
        ),
        indoc!(
            r#"
                🌏
            "#,
        ),
        indoc!(
            r#"
                🌍
            "#,
        ),
    ]
}
