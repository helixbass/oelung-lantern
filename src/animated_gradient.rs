use crossterm::style::Color;
use oelung::{anyhow, Component, ComponentInterface, FlexColumnBuilder, Grid, TextBuilder};
use palette::{Luv, Mix};

use crate::{
    animation, mpsc::Sender, to_color, to_luv, Animation, AnimationInstance, AnimationRepeat,
    Interpolateable,
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

    fn current_start_color(&self) -> Luv {
        self.start_color
            .interpolate(&self.finish_start_color, self.animation.current_progress())
    }

    fn current_end_color(&self) -> Luv {
        self.end_color
            .interpolate(&self.finish_end_color, self.animation.current_progress())
    }

    fn get_intermediate_color(&self, step_num: u16) -> Luv {
        self.current_start_color().mix(
            self.current_end_color(),
            if self.width > 1 {
                step_num as f32 / (self.width - 1) as f32
            } else {
                0.0
            },
        )
    }

    fn render_row<'b>(&self) -> Component<'b> {
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

impl<'a> ComponentInterface for &'a AnimatedGradient {
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

pub type Event = animation::Event;
