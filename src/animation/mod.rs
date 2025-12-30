use std::time::Duration;

use derive_builder::Builder;

pub enum AnimationRepeat {
    ForwardOnce,
    ForwardInfinite,
    ForwardNTimes(u32),
    ForwardAndBackOnce,
    ForwardAndBackInfinite,
    ForwardAndBackNTimes(u32),
}

#[derive(Builder, Clone)]
pub struct Animation {
    pub duration: Duration,
    pub easing: Easing,
}

#[derive(Clone)]
pub enum Easing {
    Linear,
}

impl Easing {
    pub fn interpolate(&self, progress: f32) -> f32 {
        assert!(progress >= 0.0 && progress <= 1.0);
        match self {
            Self::Linear => progress,
        }
    }
}

pub trait Interpolateable {
    fn interpolate(start: &Self, end: &Self, progress: f32) -> Self;
}
