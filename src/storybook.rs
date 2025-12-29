use std::collections::HashMap;
use std::time::Duration;

use crossterm::style::Color;
use smol_str::SmolStr;
use tokio::sync::mpsc;

pub struct Storybook {}

pub struct StorybookBuilder {}

pub enum InputValue {
    Color(Color),
    Duration(Duration),
}

impl InputValue {
    pub fn as_color(&self) -> &Color {
        match self {
            Self::Color(color) => color,
            _ => panic!("expected color"),
        }
    }

    pub fn as_duration(&self) -> &Duration {
        match self {
            Self::Duration(duration) => duration,
            _ => panic!("expected duration"),
        }
    }
}

pub struct Input {
    name: SmolStr,
    input_type: InputType,
    is_optional: bool,
}

impl Input {
    pub fn new(name: impl Into<SmolStr>, input_type: InputType, is_optional: bool) -> Self {
        Self {
            name: name.into(),
            input_type,
            is_optional,
        }
    }
}

pub enum InputType {
    Color,
    Duration,
}

pub trait Component<TWorld> {
    fn name(&self) -> SmolStr;
    fn inputs(&self) -> Vec<Input>;
    fn get_component(
        &self,
        inputs: &HashMap<SmolStr, Option<InputValue>>,
        sender: mpsc::Sender<TWorld>,
    ) -> Box<dyn ComponentInstance<TWorld>>;
}

pub trait ComponentInstance<TWorld> {
    fn get_component<'b>(&self) -> oelung::Component<'b>;
    fn receive(&mut self, event: &TWorld);
}
