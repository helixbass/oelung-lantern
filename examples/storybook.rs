use std::collections::HashMap;

use crossterm::{
    event::{Event, EventStream, KeyCode},
    style::Color,
};
use oelung::{soft, Component, Renderer};
use smol_str::{SmolStr, ToSmolStr};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung_lantern::{generate_sender, mpsc::Sender, spinner, storybook, StorybookBuilder};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let storybook = StorybookBuilder::default()
        .components(vec![
            storybook::ComponentBuilder::default()
                .name("Snake spinner")
                .
        ])
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

fn render_screen(renderer: &mut Renderer, gradient: &Gradient) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %gradient
          %Text "(hit q to quit)"
        ]
    })?;

    Ok(())
}

enum World {
    Crossterm(Event),
}

generate_sender!(World, Crossterm, Event);

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

impl storybook::Component for SnakeSpinner {
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
    ) -> Box<dyn storybook::ComponentInstance> {
        let period = inputs["period"].as_ref().map(|input| input.as_duration());
        let color = inputs["color"].as_ref().map(|input| input.as_color());

        spinner::SnakeSpinner::new(period, color)
    }
}

enum World {
    Crossterm(Event),
}
