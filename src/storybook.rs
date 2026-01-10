use std::collections::HashMap;
use std::marker::PhantomData;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use crossterm::{
    event::{self, KeyCode, KeyEvent},
    style::Color,
};
use oelung::{anyhow, soft, ComponentInterface, Grid};
use smol_str::SmolStr;
use squalid::_d;
use tracing::instrument;

use crate::{
    is_any_simple_char_press, is_any_simple_char_press_key_event, is_ctrl_char_press,
    is_simple_key_press, is_simple_key_press_key_event, mpsc::Sender, Error, ReceiveEvent,
};

pub struct Storybook<TWorld> {
    pub components: Vec<Box<dyn Component<TWorld>>>,
    pub currently_selected_component: Option<Box<dyn ComponentInstance<TWorld>>>,
    pub current_inputs: Option<Vec<InputInstance>>,
    pub sender: Box<dyn Sender<TWorld>>,
    pub storybook_event_from: Box<dyn StorybookEventFrom<TWorld>>,
    pub mode: Mode<TWorld>,
}

pub struct StorybookBuilder<TWorld> {
    pub components: Option<Vec<Box<dyn Component<TWorld>>>>,
    pub sender: Option<Box<dyn Sender<TWorld>>>,
    pub storybook_event_from: Option<Box<dyn StorybookEventFrom<TWorld>>>,
}

impl<TWorld> Default for StorybookBuilder<TWorld> {
    fn default() -> Self {
        Self {
            components: _d(),
            sender: _d(),
            storybook_event_from: _d(),
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

    pub fn storybook_event_from(
        mut self,
        storybook_event_from: Box<dyn StorybookEventFrom<TWorld>>,
    ) -> Self {
        self.storybook_event_from = Some(storybook_event_from);
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
            storybook_event_from: self.storybook_event_from.ok_or_else(|| {
                Error::StorybookBuilder("expected storybook_event_from".to_owned())
            })?,
            mode: _d(),
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

    pub fn receive_storybook_event<'a>(
        &mut self,
        event: Event,
        queue_effect: Box<dyn FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + 'a>,
    ) -> Result<(), anyhow::Error> {
        match (&self.mode, event) {
            (Mode::Normal, Event::OpenComponentChooser) => {
                self.mode = Mode::ComponentChooser(_d());
            }
            (_, Event::GoIntoNormalMode) => {
                self.mode = Mode::Normal;
            }
            (Mode::ComponentChooser(component_chooser), Event::ChooseComponent) => {
                if component_chooser.indices.is_empty() {
                    return Ok(());
                }
                self.select_component(component_chooser.indices[0]);
                self.mode = Mode::Normal;
                self.storybook_event_from.receive_update_aggregator_state(
                    UpdateAggregatorState::Initial,
                    queue_effect,
                )?;
            }
            (Mode::ComponentChooser(_), Event::ComponentChooserKey(key_event)) => {
                self.mode
                    .as_component_chooser_mut()
                    .receive_key_event(&key_event, &self.components);
            }
            _ => panic!("unexpected event"),
        }

        Ok(())
    }
}

impl<'a, TWorld> ComponentInterface for &'a Storybook<TWorld> {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render(&self, _grid: Grid) -> Result<oelung::Component<'_>, anyhow::Error> {
        Ok(match &self.mode {
            Mode::Normal => soft! {
                %FlexRow children => [
                  %ComponentPanel::new(
                      self.currently_selected_component.as_ref().unwrap().get_component()
                  )
                ]
            },
            Mode::ComponentChooser(component_chooser) => soft! {
                %FlexRow children => [
                  %ComponentChooserView::new(
                      component_chooser,
                      &self.components,
                  )
                ]
            },
        })
    }

    fn flex_grow(&self) -> Option<f64> {
        Some(1.0)
    }
}

impl<TWorld> ReceiveEvent<TWorld> for Storybook<TWorld> {
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &TWorld,
        mut queue_effect: TQueueEffect,
    ) -> Result<(), anyhow::Error> {
        if let Some(storybook_event) = self
            .storybook_event_from
            .get(event, Box::new(&mut queue_effect))?
        {
            self.receive_storybook_event(storybook_event, Box::new(&mut queue_effect))?;
        } else if let Some(currently_selected_component) =
            self.currently_selected_component.as_mut()
        {
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

pub trait StorybookEventFrom<TWorld> {
    fn get<'a>(
        &mut self,
        event: &TWorld,
        queue_effect: Box<dyn FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + 'a>,
    ) -> Result<Option<Event>, anyhow::Error>;
    fn receive_update_aggregator_state<'a>(
        &mut self,
        _update_aggregator_state: UpdateAggregatorState,
        _queue_effect: Box<dyn FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + 'a>,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

pub struct ComponentChooser<TWorld> {
    pub search: String,
    pub indices: Vec<usize>,
    phantom_data: PhantomData<TWorld>,
}

impl<TWorld> Default for ComponentChooser<TWorld> {
    fn default() -> Self {
        Self {
            search: _d(),
            indices: _d(),
            phantom_data: PhantomData,
        }
    }
}

impl<TWorld> ComponentChooser<TWorld> {
    pub fn receive_key_event(
        &mut self,
        key_event: &KeyEvent,
        components: &[Box<dyn Component<TWorld>>],
    ) {
        if let Some(ch) = is_any_simple_char_press_key_event(key_event) {
            self.search.push(ch);
            self.recompute_indices(components);
        } else if is_simple_key_press_key_event(key_event, KeyCode::Backspace) {
            let _ = self.search.pop();
            self.recompute_indices(components);
        } else {
            panic!("unexpected key event")
        }
    }

    fn recompute_indices(&mut self, components: &[Box<dyn Component<TWorld>>]) {
        self.indices = components
            .into_iter()
            .enumerate()
            .filter_map(|(index, component)| {
                component.name().starts_with(&self.search).then_some(index)
            })
            .collect();
    }
}

#[derive(Default)]
pub enum Mode<TWorld> {
    #[default]
    Normal,
    ComponentChooser(ComponentChooser<TWorld>),
}

impl<TWorld> Mode<TWorld> {
    pub fn as_component_chooser_mut(&mut self) -> &mut ComponentChooser<TWorld> {
        match self {
            Self::ComponentChooser(component_chooser) => component_chooser,
            _ => panic!("expected component chooser"),
        }
    }
}

#[derive(Debug)]
pub enum Event {
    OpenComponentChooser,
    GoIntoNormalMode,
    ComponentChooserKey(KeyEvent),
    ChooseComponent,
}

#[derive(Default)]
pub struct Aggregator {
    pub state: State,
}

#[derive(Copy, Clone, Default)]
pub enum State {
    #[default]
    Initial,
    ComponentChooser,
}

#[derive(Copy, Clone, Debug)]
pub enum UpdateAggregatorState {
    Initial,
}

impl ReceiveEvent<event::Event, Option<Event>> for Aggregator {
    #[instrument(level = "trace", skip(self, event, _queue_effect))]
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &event::Event,
        _queue_effect: TQueueEffect,
    ) -> Result<Option<Event>, anyhow::Error> {
        Ok(match (&self.state, event) {
            (State::Initial, event) if is_ctrl_char_press(event, 'c') => {
                self.state = State::ComponentChooser;
                Some(Event::OpenComponentChooser)
            }
            (_, event) if is_simple_key_press(event, KeyCode::Esc) => {
                self.state = State::Initial;
                Some(Event::GoIntoNormalMode)
            }
            (State::ComponentChooser, event)
                if is_any_simple_char_press(event).is_some()
                    || is_simple_key_press(event, KeyCode::Backspace) =>
            {
                Some(Event::ComponentChooserKey(event.as_key_event().unwrap()))
            }
            (State::ComponentChooser, event) if is_simple_key_press(event, KeyCode::Enter) => {
                Some(Event::ChooseComponent)
            }
            _ => None,
        })
    }
}

impl ReceiveEvent<UpdateAggregatorState> for Aggregator {
    #[instrument(level = "trace", skip(self, event, _queue_effect))]
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &UpdateAggregatorState,
        _queue_effect: TQueueEffect,
    ) -> Result<(), anyhow::Error> {
        Ok(match event {
            UpdateAggregatorState::Initial => {
                self.state = State::Initial;
            }
        })
    }
}

pub struct ComponentPanel<'a> {
    pub component: oelung::Component<'a>,
}

impl<'a> ComponentPanel<'a> {
    pub fn new(component: oelung::Component<'a>) -> Self {
        Self { component }
    }
}

impl<'a> ComponentInterface for ComponentPanel<'a> {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render(&self, _grid: Grid) -> Result<oelung::Component<'_>, anyhow::Error> {
        Ok(soft! {
            self.component.clone()
        })
    }

    fn flex_grow(&self) -> Option<f64> {
        Some(1.0)
    }
}

pub struct ComponentChooserView<'a, TWorld> {
    pub component_chooser: &'a ComponentChooser<TWorld>,
    pub components: &'a [Box<dyn Component<TWorld>>],
}

impl<'a, TWorld> ComponentChooserView<'a, TWorld> {
    pub fn new(
        component_chooser: &'a ComponentChooser<TWorld>,
        components: &'a [Box<dyn Component<TWorld>>],
    ) -> Self {
        Self {
            component_chooser,
            components,
        }
    }
}

impl<'a, TWorld> ComponentInterface for ComponentChooserView<'a, TWorld> {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render(&self, _grid: Grid) -> Result<oelung::Component<'_>, anyhow::Error> {
        Ok(soft! {
            %FlexColumn
              children => [
                %ComponentChooserInput::new(&self.component_chooser.search)
                %ComponentChooserResults::new(
                    self.component_chooser,
                    self.components,
                )
              ]
        })
    }

    fn flex_grow(&self) -> Option<f64> {
        Some(1.0)
    }
}

pub struct ComponentChooserInput<'a> {
    pub search: &'a str,
}

impl<'a> ComponentChooserInput<'a> {
    pub fn new(search: &'a str) -> Self {
        Self { search }
    }
}

impl<'a> ComponentInterface for ComponentChooserInput<'a> {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render(&self, _grid: Grid) -> Result<oelung::Component<'_>, anyhow::Error> {
        Ok(soft! {
            %Text self.search
        })
    }
}

pub struct ComponentChooserResults<'a, TWorld> {
    pub component_chooser: &'a ComponentChooser<TWorld>,
    pub components: &'a [Box<dyn Component<TWorld>>],
}

impl<'a, TWorld> ComponentChooserResults<'a, TWorld> {
    pub fn new(
        component_chooser: &'a ComponentChooser<TWorld>,
        components: &'a [Box<dyn Component<TWorld>>],
    ) -> Self {
        Self {
            component_chooser,
            components,
        }
    }
}

impl<'a, TWorld> ComponentInterface for ComponentChooserResults<'a, TWorld> {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render(&self, _grid: Grid) -> Result<oelung::Component<'_>, anyhow::Error> {
        Ok(match self.component_chooser.indices.is_empty() {
            false => soft! {
                %FlexColumn
                  children =>
                    self
                        .component_chooser
                        .indices
                        .iter()
                        .map(|index| &self.components[*index])
                        .map(|component| -> Result<_, anyhow::Error> {
                            Ok(soft! {
                                %Text component.name()
                            })
                        })
                        .collect::<Result<_, _>>()?
            },
            true => soft! {
                %Text "no matches"
            },
        })
    }
}
