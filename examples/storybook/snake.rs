use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use crossterm::style::Color;
use oelung::Component;
use smol_str::{SmolStr, ToSmolStr};

use oelung_lantern::{
    mpsc::Sender,
    spinner::{self, snake},
    storybook, ReceiveEvent,
};

use super::{SnakeSpinnerTickSender, World};

pub struct SnakeSpinner {}

impl SnakeSpinner {
    pub fn new() -> Self {
        Self {}
    }
}

impl storybook::Component<World> for SnakeSpinner {
    fn name(&self) -> SmolStr {
        "Snake spinner".to_smolstr()
    }

    fn inputs(&self) -> Vec<Rc<storybook::Input>> {
        vec![
            Rc::new(storybook::Input::new(
                "period",
                storybook::InputType::Duration,
                storybook::InputValue::Duration(Duration::from_millis(snake::DEFAULT_PERIOD)),
            )),
            Rc::new(storybook::Input::new(
                "color",
                storybook::InputType::Color,
                storybook::InputValue::Color(Color::Reset),
            )),
        ]
    }

    fn get_component(
        &self,
        inputs: &HashMap<SmolStr, storybook::InputValue>,
        sender: Box<dyn Sender<World>>,
    ) -> Box<dyn storybook::ComponentInstance<World>> {
        let period = inputs["period"].as_duration();
        let color = inputs["color"].as_color();

        Box::new(spinner::SnakeSpinner::new(
            Some(period.clone()),
            Some(color.clone()),
            Box::new(SnakeSpinnerTickSender::from(sender)),
        ))
    }
}

impl storybook::ComponentInstance<World> for spinner::SnakeSpinner {
    fn get_component(&self) -> Component<'_> {
        Component::Component(Rc::new(self))
    }

    fn receive<'a>(
        &mut self,
        event: &World,
        mut queue_effect: Box<dyn FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + 'a>,
    ) -> Result<(), anyhow::Error> {
        Ok(match event {
            World::SnakeSpinnerTick(tick) => {
                ReceiveEvent::<snake::Tick>::receive(self, tick, &mut queue_effect)?;
            }
            _ => {}
        })
    }
}
