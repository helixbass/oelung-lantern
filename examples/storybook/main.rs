use std::pin::Pin;

use crossterm::event::{Event, EventStream, KeyCode};
use oelung::{soft, Renderer, RendererBuilder};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung_lantern::{
    generate_full_sender, generate_sender, generate_sender_from_sender, mpsc::Sender, spinner,
    ReceiveEvent, Storybook, StorybookBuilder,
};

mod snake;

use snake::SnakeSpinner;

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

enum World {
    Crossterm(Event),
    SnakeSpinnerTick(spinner::snake::Tick),
}

generate_sender!(World, Crossterm, Event);
generate_sender_from_sender!(World, SnakeSpinnerTick, spinner::snake::Tick);
generate_full_sender!(World);
