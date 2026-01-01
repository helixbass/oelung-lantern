use std::pin::Pin;

use crossterm::style::Color;
use oelung::{anyhow, Component, ComponentInterface, FlexColumnBuilder, Grid, TextBuilder};
use palette::{Luv, Mix};
use tracing::instrument;

use crate::{
    animation, mpsc::Sender, to_color, to_luv, Animation, AnimationInstance, AnimationRepeat,
    Interpolateable, ReceiveEvent,
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

impl AnimatedGradient {
    pub fn new(
        start_color: Color,
        end_color: Color,
        finish_start_color: Color,
        finish_end_color: Color,
        height: u16,
        width: u16,
        animation_repeat: AnimationRepeat,
        animation: Animation,
        sender: Box<dyn Sender<Event>>,
    ) -> Self {
        Self {
            start_color: to_luv(start_color),
            end_color: to_luv(end_color),
            finish_start_color: to_luv(finish_start_color),
            finish_end_color: to_luv(finish_end_color),
            height,
            width,
            animation: AnimationInstance::new(animation_repeat, animation, sender),
        }
    }

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
