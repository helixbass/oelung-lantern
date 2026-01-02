use std::pin::Pin;

use crossterm::style::Color;
use oelung::{anyhow, Component, ComponentInterface, FlexColumnBuilder, Grid, TextBuilder};
use palette::{Luv, Mix};
use squalid::OptionExt;
use tracing::instrument;

use crate::{
    animation, mpsc::Sender, to_color, to_luv, Animation, AnimationInstance, AnimationRepeat,
    Error, Interpolateable, ReceiveEvent,
};

pub struct AnimatedGradient {
    pub start_color: Luv,
    pub end_color: Luv,
    pub finish_start_color: Luv,
    pub finish_end_color: Luv,
    pub height: u16,
    pub width: u16,
    pub animation: AnimationInstance,
}

#[derive(Default)]
pub struct AnimatedGradientBuilder {
    pub start_color: Option<Luv>,
    pub end_color: Option<Luv>,
    pub finish_start_color: Option<Luv>,
    pub finish_end_color: Option<Luv>,
    pub height: Option<u16>,
    pub width: Option<u16>,
    pub animation_instance: Option<AnimationInstance>,
    pub animation_repeat: Option<AnimationRepeat>,
    pub animation: Option<Animation>,
    pub sender: Option<Box<dyn Sender<Event>>>,
}

impl AnimatedGradientBuilder {
    pub fn start_color(mut self, start_color: Color) -> Self {
        self.start_color = Some(to_luv(start_color));
        self
    }

    pub fn end_color(mut self, end_color: Color) -> Self {
        self.end_color = Some(to_luv(end_color));
        self
    }

    pub fn finish_start_color(mut self, finish_start_color: Color) -> Self {
        self.finish_start_color = Some(to_luv(finish_start_color));
        self
    }

    pub fn finish_end_color(mut self, finish_end_color: Color) -> Self {
        self.finish_end_color = Some(to_luv(finish_end_color));
        self
    }

    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    pub fn animation_instance(mut self, animation_instance: AnimationInstance) -> Self {
        if self.animation_repeat.is_some() || self.animation.is_some() || self.sender.is_some() {
            panic!("Can't use both `.animation_instance()` and individual `.animation_repeat()`/`.animation()`/`.sender()` methods");
        }
        self.animation_instance = Some(animation_instance);
        self
    }

    pub fn animation(mut self, animation: Animation) -> Self {
        if self.animation_instance.is_some() {
            panic!("Can't use both `.animation_instance()` and individual `.animation_repeat()`/`.animation()`/`.sender()` methods");
        }
        self.animation = Some(animation);
        self
    }

    pub fn animation_repeat(mut self, animation_repeat: AnimationRepeat) -> Self {
        if self.animation_instance.is_some() {
            panic!("Can't use both `.animation_instance()` and individual `.animation_repeat()`/`.animation()`/`.sender()` methods");
        }
        self.animation_repeat = Some(animation_repeat);
        self
    }

    pub fn sender(mut self, sender: Box<dyn Sender<Event>>) -> Self {
        if self.animation_instance.is_some() {
            panic!("Can't use both `.animation_instance()` and individual `.animation_repeat()`/`.animation()`/`.sender()` methods");
        }
        self.sender = Some(sender);
        self
    }

    pub fn is_height_set(&self) -> bool {
        self.height.is_some()
    }

    pub fn is_width_set(&self) -> bool {
        self.width.is_some()
    }

    pub fn build(self) -> Result<AnimatedGradient, Error> {
        Ok(AnimatedGradient {
            start_color: self.start_color.ok_or_else(|| {
                Error::AnimatedGradientBuilder("expected `start_color`".to_owned())
            })?,
            end_color: self
                .end_color
                .ok_or_else(|| Error::AnimatedGradientBuilder("expected `end_color`".to_owned()))?,
            finish_start_color: self.finish_start_color.ok_or_else(|| {
                Error::AnimatedGradientBuilder("expected `finish_start_color`".to_owned())
            })?,
            finish_end_color: self.finish_end_color.ok_or_else(|| {
                Error::AnimatedGradientBuilder("expected `finish_end_color`".to_owned())
            })?,
            height: self
                .height
                .ok_or_else(|| Error::AnimatedGradientBuilder("expected `height`".to_owned()))?,
            width: self
                .width
                .ok_or_else(|| Error::AnimatedGradientBuilder("expected `height`".to_owned()))?,
            animation: self.animation_instance.try_or_else(|| -> Result<_, Error> {
                if !(self.animation.is_some() || self.animation_repeat.is_some() || self.sender.is_some()) {
                    return Ok(None);
                }
                Ok(Some(AnimationInstance::new(
                    self
                        .animation_repeat
                        .ok_or_else(|| Error::AnimatedGradientBuilder("expected `animation_repeat`".to_owned()))?,
                    self
                        .animation
                        .ok_or_else(|| Error::AnimatedGradientBuilder("expected `animation`".to_owned()))?,
                    self
                        .sender
                        .ok_or_else(|| Error::AnimatedGradientBuilder("expected `sender`".to_owned()))?,
                )))
            })?
            .ok_or_else(|| Error::AnimatedGradientBuilder("expected `animation_instance` or `animation` + `sender` + `animation_repeat`".to_owned()))?,
        })
    }
}

impl AnimatedGradient {
    // #[instrument(level = "trace", skip(self))]
    fn current_start_color(&self) -> Luv {
        self.start_color
            .interpolate(&self.finish_start_color, self.animation.current_progress())
    }

    // #[instrument(level = "trace", skip(self))]
    fn current_end_color(&self) -> Luv {
        self.end_color
            .interpolate(&self.finish_end_color, self.animation.current_progress())
    }

    #[instrument(level = "trace", skip(self))]
    fn render_row<'a>(&'a self, steps: &[Color]) -> Component<'a> {
        let mut text = TextBuilder::default();
        for step in steps {
            text = text.nested_child(
                TextBuilder::default()
                    .background_color(*step)
                    .text_child(" ")
                    .build()
                    .unwrap(),
            );
        }
        text.build().unwrap().into()
    }
}

impl<'a> ComponentInterface for &'a AnimatedGradient {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render(&self, _grid: Grid) -> Result<Component<'_>, anyhow::Error> {
        Ok({
            let current_start_color = self.current_start_color();
            let current_end_color = self.current_end_color();
            let steps = (0..self.width)
                .into_iter()
                .map(|step_num| {
                    to_color(get_intermediate_color(
                        current_start_color,
                        current_end_color,
                        self.width,
                        step_num,
                    ))
                })
                .collect::<Vec<_>>();
            if self.height > 1 {
                let mut flex_column = FlexColumnBuilder::default();
                for _ in 0..self.height {
                    flex_column = flex_column.child(self.render_row(&steps));
                }
                flex_column.build().unwrap().into()
            } else {
                self.render_row(&steps)
            }
        })
    }

    fn height(&self) -> Option<u16> {
        Some(self.height)
    }
}

pub type Event = animation::Event;

impl ReceiveEvent<animation::Tick> for AnimatedGradient {
    #[instrument(level = "trace", skip(self, tick, queue_effect))]
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        tick: &animation::Tick,
        queue_effect: TQueueEffect,
    ) {
        self.animation.receive(tick, queue_effect);
    }
}

// #[instrument(level = "trace", skip(self))]
fn get_intermediate_color(start_color: Luv, end_color: Luv, num_steps: u16, step_num: u16) -> Luv {
    start_color.mix(
        end_color,
        if num_steps > 1 {
            step_num as f32 / (num_steps - 1) as f32
        } else {
            0.0
        },
    )
}
