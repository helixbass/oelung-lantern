use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;

use crossterm::style::Color;
use oelung::{anyhow, ComponentInterface, Grid};
use smol_str::SmolStr;
use squalid::_d;
use tokio::sync::mpsc;
use tracing::instrument;

use crate::{Error, ReceiveEvent};

pub struct Storybook<TWorld> {
    pub components: Vec<Box<dyn Component<TWorld>>>,
    pub currently_selected_component: Option<Box<dyn ComponentInstance<TWorld>>>,
}

pub struct StorybookBuilder<TWorld> {
    pub components: Option<Vec<Box<dyn Component<TWorld>>>>,
}

impl<TWorld> Default for StorybookBuilder<TWorld> {
    fn default() -> Self {
        Self { components: _d() }
    }
}

impl<TWorld> StorybookBuilder<TWorld> {
    pub fn components(mut self, components: Vec<Box<dyn Component<TWorld>>>) -> Self {
        self.components = Some(components);
        self
    }

    pub fn build(self) -> Result<Storybook<TWorld>, Error> {
        Ok(Storybook {
            components: self
                .components
                .ok_or_else(|| Error::StorybookBuilder("expected components".to_owned()))?,
            currently_selected_component: _d(),
        })
    }
}

impl<'a, TWorld> ComponentInterface for &'a Storybook<TWorld> {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render(&self, _grid: Grid) -> Result<oelung::Component<'_>, anyhow::Error> {
        unimplemented!()
    }
}

impl<TWorld> ReceiveEvent<TWorld> for Storybook<TWorld> {
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &TWorld,
        queue_effect: TQueueEffect,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }
}

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
    fn get_component(&self) -> oelung::Component<'_>;
    fn receive(
        &mut self,
        event: &TWorld,
        queue_effect: Box<dyn FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>,
    ) -> Result<(), anyhow::Error>;
}
