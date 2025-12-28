use std::time::Duration;

use crossterm::{
    event::{self, Event, EventStream, KeyCode},
    style::Color,
};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung::{soft, ComponentInterface, Renderer};

use oelung_lantern::{generate_sender, mpsc, spinner::snake, ReceiveEvent, SnakeSpinner};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let spinner = SnakeSpinner::new(
        Duration::from_millis(1000),
        Some(Color::Red),
        Box::new(SnakeSpinnerTickSender::from(sender)),
    );

    render_screen(&mut renderer, &spinner)?;

    while let Some(world) = receiver.recv().await {
        match world {
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            World::SnakeSpinnerTick(tick) => {
                spinner.receive(&tick);
                render_screen(&mut renderer, &spinner);
            }
            _ => {}
        }
    }

    Ok(())
}

fn render_screen(renderer: &mut Renderer, spinner: &SnakeSpinner) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %Text
        children => [
          spinner
          %Text " (hit q to quit)"
        ]
    })?;

    Ok(())
}

enum World {
    Crossterm(Event),
    SnakeSpinnerTick(snake::Tick),
}

generate_sender!(World, Crossterm, Event);
generate_sender!(World, SnakeSpinnerTick, snake::Tick);

fn listen_to_crossterm_events(sender: CrosstermSender) {
    tokio::spawn(async move {
        let mut event_stream = EventStream::new();

        while let Some(Ok(event)) = event_stream.next().await {
            sender.send(event).await;
        }

        panic!("kill everything")
    });
}
