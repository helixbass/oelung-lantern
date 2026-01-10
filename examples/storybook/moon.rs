use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use oelung::Component;
use smol_str::{SmolStr, ToSmolStr};

use oelung_lantern::{
    mpsc::Sender,
    spinner::{self, moon},
    storybook, ReceiveEvent,
};

use super::{MoonSpinnerTickSender, World};

pub struct MoonSpinner {}

impl MoonSpinner {
    pub fn new() -> Self {
        Self {}
    }
}

impl storybook::Component<World> for MoonSpinner {
    fn name(&self) -> SmolStr {
        "Moon spinner".to_smolstr()
    }

    fn inputs(&self) -> Vec<Rc<storybook::Input>> {
        vec![Rc::new(storybook::Input::new(
            "period",
            storybook::InputType::Duration,
            storybook::InputValue::Duration(Duration::from_millis(1000)),
        ))]
    }

    fn get_component(
        &self,
        inputs: &HashMap<SmolStr, storybook::InputValue>,
        sender: Box<dyn Sender<World>>,
    ) -> Box<dyn storybook::ComponentInstance<World>> {
        let period = inputs["period"].as_duration();

        Box::new(spinner::MoonSpinner::new(
            Some(period.clone()),
            Box::new(MoonSpinnerTickSender::from(sender)),
        ))
    }
}

impl storybook::ComponentInstance<World> for spinner::MoonSpinner {
    fn get_component(&self) -> Component<'_> {
        Component::Component(Rc::new(self))
    }

    fn receive<'a>(
        &mut self,
        event: &World,
        mut queue_effect: Box<dyn FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + 'a>,
    ) -> Result<(), anyhow::Error> {
        Ok(match event {
            World::MoonSpinnerTick(tick) => {
                ReceiveEvent::<moon::Tick>::receive(self, tick, &mut queue_effect)?;
            }
            _ => {}
        })
    }
}
