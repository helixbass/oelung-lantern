use crossterm::style::Color;
use indoc::indoc;
use oelung_lantern::{render_multiple_test, spinner::ellipsis, EllipsisSpinner};

render_multiple_test! {
    name => ellipsis_spinner
    state => |sender| EllipsisSpinner::new(
        None,
        Some(Color::Red),
        sender
    )
    state_type => EllipsisSpinner
    send_and_receive => ellipsis::Tick
    expected_prefix => vec![
        indoc!(
            r#"
                <color={Red}>∙∙∙</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>●∙∙</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>∙●∙</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>∙∙●</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>∙∙∙</>
            "#,
        ),
        indoc!(
            r#"
                <color={Red}>∙∙∙</>
            "#,
        ),
    ]
}
