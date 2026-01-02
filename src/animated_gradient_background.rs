use oelung::{anyhow, soft, Component, ComponentInterface, Grid, Relative};

use crate::{AnimatedGradient, AnimatedGradientBuilder};

pub struct AnimatedGradientBackground<'a> {
    pub gradient: AnimatedGradient,
    pub content: Component<'a>,
    pub width: u16,
}

impl<'a> AnimatedGradientBackground<'a> {
    pub fn new(gradient: AnimatedGradientBuilder, content: Component<'a>, width: u16) -> Self {
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

impl<'a> ComponentInterface for AnimatedGradientBackground<'a> {
    fn render(&self, _grid: Grid) -> Result<Component<'_>, anyhow::Error> {
        Ok(soft! {
            %FlexColumn
              children => [
                %Absolute
                  content => soft! { %&self.gradient }
                %Absolute
                  content => self.content.clone()
              ]
              relative => Relative::NotMoved
        })
    }

    fn height(&self) -> Option<u16> {
        self.content.height()
    }
}
