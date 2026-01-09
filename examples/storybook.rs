use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use crossterm::{
    event::{Event, EventStream, KeyCode},
    style::Color,
};
use oelung::{soft, Component, Renderer, RendererBuilder};
use smol_str::{SmolStr, ToSmolStr};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung_lantern::{
    generate_full_sender, generate_sender, generate_sender_from_sender,
    mpsc::Sender,
    spinner::{self, snake},
    storybook, ReceiveEvent, Storybook, StorybookBuilder,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = RendererBuilder::default().build()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let mut storybook = StorybookBuilder::default()
        .components(vec![Box::new(SnakeSpinner::new())])
        .sender(Box::new(WorldSender::from(sender.clone())))
        .build()
        .unwrap();
    storybook.select_component(0);

    render_screen(&mut renderer, &storybook)?;

    while let Some(world) = receiver.recv().await {
        let mut queued_effects: Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> = vec![];
        match world {
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            event => {
                storybook.receive(&event, |future| queued_effects.push(future))?;
                render_screen(&mut renderer, &storybook)?;
            }
        }
        for effect in queued_effects {
            tokio::spawn(effect);
        }
    }

    Ok(())
}

fn render_screen(
    renderer: &mut Renderer,
    storybook: &Storybook<World>,
) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %storybook
          %Text "(hit q to quit)"
        ]
    })?;

    Ok(())
}

fn listen_to_crossterm_events(sender: CrosstermSender) {
    tokio::spawn(async move {
        let mut event_stream = EventStream::new();

        while let Some(Ok(event)) = event_stream.next().await {
            sender.send(event).await;
        }

        panic!("kill everything")
    });
}

struct SnakeSpinner {}

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
                storybook::InputValue::Duration(Duration::from_millis(1000)),
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

enum World {
    Crossterm(Event),
    SnakeSpinnerTick(snake::Tick),
}

generate_sender!(World, Crossterm, Event);
generate_sender_from_sender!(World, SnakeSpinnerTick, snake::Tick);
generate_full_sender!(World);
