use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use crossterm::style::Color;
use oelung::{anyhow, ComponentInterface, Grid};
use smol_str::SmolStr;
use squalid::_d;
use tracing::instrument;

use crate::{mpsc::Sender, Error, ReceiveEvent};

pub struct Storybook<TWorld> {
    pub components: Vec<Box<dyn Component<TWorld>>>,
    pub currently_selected_component: Option<Box<dyn ComponentInstance<TWorld>>>,
    pub current_inputs: Option<Vec<InputInstance>>,
    pub sender: Box<dyn Sender<TWorld>>,
}

pub struct StorybookBuilder<TWorld> {
    pub components: Option<Vec<Box<dyn Component<TWorld>>>>,
    pub sender: Option<Box<dyn Sender<TWorld>>>,
}

impl<TWorld> Default for StorybookBuilder<TWorld> {
    fn default() -> Self {
        Self {
            components: _d(),
            sender: _d(),
        }
    }
}

impl<TWorld> StorybookBuilder<TWorld> {
    pub fn components(mut self, components: Vec<Box<dyn Component<TWorld>>>) -> Self {
        self.components = Some(components);
        self
    }

    pub fn sender(mut self, sender: Box<dyn Sender<TWorld>>) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn build(self) -> Result<Storybook<TWorld>, Error> {
        Ok(Storybook {
            components: self
                .components
                .ok_or_else(|| Error::StorybookBuilder("expected components".to_owned()))?,
            currently_selected_component: _d(),
            current_inputs: _d(),
            sender: self
                .sender
                .ok_or_else(|| Error::StorybookBuilder("expected sender".to_owned()))?,
        })
    }
}

impl<TWorld> Storybook<TWorld> {
    pub fn select_component(&mut self, index: usize) {
        self.current_inputs = Some(
            self.components[index]
                .inputs()
                .into_iter()
                .map(|input| InputInstance {
                    value: input.default_value.clone(),
                    input,
                })
                .collect(),
        );
        self.currently_selected_component = Some(
            self.components[index]
                .get_component(&self.current_input_values(), self.sender.box_clone()),
        );
    }

    pub fn current_input_values(&self) -> HashMap<SmolStr, InputValue> {
        self.current_inputs
            .as_ref()
            .unwrap()
            .into_iter()
            .map(|input| (input.input.name.clone(), input.value.clone()))
            .collect()
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
        if let Some(currently_selected_component) = self.currently_selected_component.as_mut() {
            currently_selected_component.receive(event, Box::new(queue_effect))?;
        }

        Ok(())
    }
}

#[derive(Clone)]
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
    default_value: InputValue,
}

impl Input {
    pub fn new(name: impl Into<SmolStr>, input_type: InputType, default_value: InputValue) -> Self {
        Self {
            name: name.into(),
            input_type,
            default_value,
        }
    }
}

pub enum InputType {
    Color,
    Duration,
}

pub struct InputInstance {
    pub input: Rc<Input>,
    pub value: InputValue,
}

pub trait Component<TWorld> {
    fn name(&self) -> SmolStr;
    fn inputs(&self) -> Vec<Rc<Input>>;
    fn get_component(
        &self,
        inputs: &HashMap<SmolStr, InputValue>,
        sender: Box<dyn Sender<TWorld>>,
    ) -> Box<dyn ComponentInstance<TWorld>>;
}

pub trait ComponentInstance<TWorld> {
    fn get_component(&self) -> oelung::Component<'_>;
    fn receive<'a>(
        &mut self,
        event: &TWorld,
        queue_effect: Box<dyn FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + 'a>,
    ) -> Result<(), anyhow::Error>;
}
