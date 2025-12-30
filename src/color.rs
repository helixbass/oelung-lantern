use crossterm::style::Color;
use palette::{FromColor, Luv, Mix, Srgb};
use squalid::EverythingExt;
use tracing::instrument;

use crate::Interpolateable;

impl Interpolateable for Luv {
    #[instrument(level = "trace", skip(self, other, progress))]
    fn interpolate(&self, other: &Self, progress: f32) -> Self {
        self.mix(*other, progress)
    }
}

#[instrument(level = "trace", skip(color))]
pub fn to_luv(color: Color) -> Luv {
    match color {
        Color::Rgb { r, g, b } => Luv::from_color(Srgb::new(r, g, b).into_format()),
        _ => unimplemented!(),
    }
}

#[instrument(level = "trace", skip(luv))]
pub fn to_color(luv: Luv) -> Color {
    Srgb::from_color(luv)
        .into_format::<u8>()
        .thrush(|rgb| Color::Rgb {
            r: rgb.red,
            g: rgb.green,
            b: rgb.blue,
        })
}
