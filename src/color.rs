use palette::{Luv, Mix};

use crate::Interpolateable;

impl Interpolateable for Luv {
    fn interpolate(start: &Self, end: &Self, progress: f32) -> Self {
        start.mix(*end, progress)
    }
}
