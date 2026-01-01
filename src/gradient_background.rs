use oelung::{anyhow, soft, Component, ComponentInterface, Grid, Relative};

use crate::{Gradient, GradientBuilder};

pub struct GradientBackground<'a> {
    pub gradient: Gradient,
    pub content: Component<'a>,
    pub width: u16,
}

impl<'a> GradientBackground<'a> {
    pub fn new(mut gradient: GradientBuilder, content: Component<'a>, width: u16) -> Self {
        assert!(!gradient.is_height_set());
        assert!(!gradient.is_width_set());

        assert!(content.height().is_some());

        Self {
            gradient: gradient
                .height(content.height().unwrap())
                .width(width)
                .build()
                .unwrap(),
            content,
            width,
        }
    }
}

impl<'a> ComponentInterface for GradientBackground<'a> {
    fn render(&self, _grid: Grid) -> Result<Component<'_>, anyhow::Error> {
        Ok(soft! {
            %FlexColumn
              children => [
                %Absolute
                  content => soft! { %&self.gradient }
                self.content.clone()
              ]
              relative => Relative::NotMoved
        })
    }

    fn height(&self) -> Option<u16> {
        self.content.height()
    }
}
