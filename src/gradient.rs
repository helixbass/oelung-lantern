use crossterm::style::Color;
use oelung::{anyhow, Component, ComponentInterface, FlexColumnBuilder, Grid, TextBuilder};
use palette::{Luv, Mix};
use tracing::instrument;

use crate::{to_color, to_luv};

pub struct Gradient {
    pub start_color: Luv,
    pub end_color: Luv,
    pub height: u16,
    pub width: u16,
}

impl Gradient {
    pub fn new(start_color: Color, end_color: Color, height: u16, width: u16) -> Self {
        Self {
            start_color: to_luv(start_color),
            end_color: to_luv(end_color),
            height,
            width,
        }
    }

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
    fn render_row(&self) -> Component<'static, 'static> {
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

impl<'a> ComponentInterface<'static> for &'a Gradient {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render(&self, _grid: Grid) -> Result<Component<'static, 'static>, anyhow::Error> {
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
