use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung::{soft, Renderer};

use oelung_lantern::{countdown, generate_sender, mpsc::Sender, Countdown};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let countdown = Countdown::new(
        Duration::from_secs(5),
        Box::new(CountdownSender::from(sender)),
    );

    render_screen(&mut renderer, &countdown)?;

    while let Some(world) = receiver.recv().await {
        match world {
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            World::Countdown(countdown::Event::Tick { .. }) => {
                render_screen(&mut renderer, &countdown)?;
            }
            // World::Countdown(countdown::Event::Done) => {
            //     render_screen(&mut renderer, &countdown)?;
            // }
            _ => {}
        }
    }

    Ok(())
}

fn render_screen(renderer: &mut Renderer, countdown: &Countdown) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %countdown
          %Text "(hit q to quit)"
        ]
    })?;

    Ok(())
}

enum World {
    Crossterm(Event),
    Countdown(countdown::Event),
}

generate_sender!(World, Crossterm, Event);
generate_sender!(World, Countdown, countdown::Event);

fn listen_to_crossterm_events(sender: CrosstermSender) {
    tokio::spawn(async move {
        let mut event_stream = EventStream::new();

        while let Some(Ok(event)) = event_stream.next().await {
            sender.send(event).await;
        }

        panic!("kill everything")
    });
}
