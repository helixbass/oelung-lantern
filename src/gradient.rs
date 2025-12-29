use crossterm::style::Color;
use oelung::{anyhow, Component, ComponentInterface, FlexColumnBuilder, Grid, TextBuilder};
use palette::{FromColor, Luv, Srgb};

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

    fn render_row<'b>(&self) -> Component<'b> {
        let mut text = TextBuilder::default();
        for step in 0..self.width {
            text = text.nested_child(
                TextBuilder::default()
                    .background_color()
                    .text_child(" ")
                    .build()
                    .unwrap(),
            );
        }
        text.build().unwrap().into()
    }
}

impl<'a> ComponentInterface for &'a Gradient {
    fn render<'b>(&self, _grid: Grid) -> Result<Component<'b>, anyhow::Error> {
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

fn to_luv(color: Color) -> Luv {
    match color {
        Color::Rgb { r, g, b } => Luv::from_color(Srgb::new(r, g, b).into_format()),
        _ => unimplemented!(),
    }
}
