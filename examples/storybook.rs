use std::collections::HashMap;
use std::rc::Rc;

use crossterm::event::{Event, EventStream, KeyCode};
use oelung::{soft, Component, Renderer};
use smol_str::{SmolStr, ToSmolStr};
use tokio::sync::mpsc::{self, channel};
use tokio_stream::StreamExt;

use oelung_lantern::{
    generate_sender,
    mpsc::Sender,
    spinner::{self, snake},
    storybook, ReceiveEvent, Storybook, StorybookBuilder,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let storybook = StorybookBuilder::default()
        .components(vec![Box::new(SnakeSpinner::new())])
        .build()
        .unwrap();

    render_screen(&mut renderer, &storybook)?;

    while let Some(world) = receiver.recv().await {
        match world {
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            _ => {}
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

    fn inputs(&self) -> Vec<storybook::Input> {
        vec![
            storybook::Input::new("period", storybook::InputType::Duration, true),
            storybook::Input::new("color", storybook::InputType::Color, true),
        ]
    }

    fn get_component(
        &self,
        inputs: &HashMap<SmolStr, Option<storybook::InputValue>>,
        sender: mpsc::Sender<World>,
    ) -> Box<dyn storybook::ComponentInstance<World>> {
        let period = inputs["period"].as_ref().map(|input| input.as_duration());
        let color = inputs["color"].as_ref().map(|input| input.as_color());

        Box::new(spinner::SnakeSpinner::new(
            period.cloned(),
            color.cloned(),
            Box::new(SnakeSpinnerTickSender::from(sender)),
        ))
    }
}

impl storybook::ComponentInstance<World> for spinner::SnakeSpinner {
    fn get_component(&self) -> Component<'_> {
        Component::Component(Rc::new(self))
    }

    fn receive(&mut self, event: &World) {
        match event {
            World::SnakeSpinnerTick(tick) => {
                // ReceiveEvent::<snake::Tick>::receive(self, tick, |_| unimplemented!());
            }
            _ => {}
        }
    }
}

enum World {
    Crossterm(Event),
    SnakeSpinnerTick(snake::Tick),
}

generate_sender!(World, Crossterm, Event);
generate_sender!(World, SnakeSpinnerTick, snake::Tick);
