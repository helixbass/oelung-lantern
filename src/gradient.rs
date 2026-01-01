use crossterm::style::Color;
use derive_builder::Builder;
use oelung::{anyhow, Component, ComponentInterface, FlexColumnBuilder, Grid, TextBuilder};
use palette::{Luv, Mix};
use tracing::instrument;

use crate::{to_color, to_luv};

#[derive(Builder)]
pub struct Gradient {
    #[builder(setter(custom))]
    pub start_color: Luv,
    #[builder(setter(custom))]
    pub end_color: Luv,
    pub height: u16,
    pub width: u16,
}

impl GradientBuilder {
    pub fn start_color(&mut self, start_color: Color) -> &mut Self {
        self.start_color = Some(to_luv(start_color));
        self
    }

    pub fn end_color(&mut self, end_color: Color) -> &mut Self {
        self.end_color = Some(to_luv(end_color));
        self
    }

    pub fn is_height_set(&self) -> bool {
        self.height.is_some()
    }

    pub fn is_width_set(&self) -> bool {
        self.width.is_some()
    }
}

impl Gradient {
    #[instrument(level = "trace", skip(self, step_num))]
    fn get_intermediate_color(&self, step_num: u16) -> Luv {
        self.start_color.mix(
            self.end_color,
            if self.width > 1 {
                step_num as f32 / (self.width - 1) as f32
            } else {
                0.0
            },
        )
    }

    #[instrument(level = "trace", skip(self))]
    fn render_row(&self) -> Component<'_> {
        let mut text = TextBuilder::default();
        for step_num in 0..self.width {
            text = text.nested_child(
                TextBuilder::default()
                    .background_color(to_color(self.get_intermediate_color(step_num)))
                    .text_child(" ")
                    .build()
                    .unwrap(),
            );
        }
        text.build().unwrap().into()
    }
}

impl<'a> ComponentInterface for &'a Gradient {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render(&self, _grid: Grid) -> Result<Component<'_>, anyhow::Error> {
        Ok({
            if self.height > 1 {
                let mut flex_column = FlexColumnBuilder::default();
                for _ in 0..self.height {
                    flex_column = flex_column.child(self.render_row());
                }
                flex_column.build().unwrap().into()
            } else {
                self.render_row()
            }
        })
    }

    fn height(&self) -> Option<u16> {
        Some(self.height)
    }
}
