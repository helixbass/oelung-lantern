use std::pin::Pin;

use crossterm::event::{Event, EventStream, KeyCode};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung::{soft, Renderer, RendererBuilder};

use oelung_lantern::{generate_sender, mpsc::Sender, spinner::moon, MoonSpinner, ReceiveEvent};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = RendererBuilder::default().build()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let mut spinner = MoonSpinner::new(None, Box::new(MoonSpinnerTickSender::from(sender)));

    render_screen(&mut renderer, &spinner)?;

    while let Some(world) = receiver.recv().await {
        let mut queued_effects: Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> = vec![];
        match world {
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            World::MoonSpinnerTick(tick) => {
                spinner.receive(&tick, |future| queued_effects.push(future))?;
                render_screen(&mut renderer, &spinner)?;
            }
            _ => {}
        }
        for effect in queued_effects {
            tokio::spawn(effect);
        }
    }

    Ok(())
}

fn render_screen(renderer: &mut Renderer, spinner: &MoonSpinner) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %Text
        children => [
          %spinner
          %Text " (hit q to quit)"
        ]
    })?;

    Ok(())
}

enum World {
    Crossterm(Event),
    MoonSpinnerTick(moon::Tick),
}

generate_sender!(World, Crossterm, Event);
generate_sender!(World, MoonSpinnerTick, moon::Tick);

fn listen_to_crossterm_events(sender: CrosstermSender) {
    tokio::spawn(async move {
        let mut event_stream = EventStream::new();

        while let Some(Ok(event)) = event_stream.next().await {
            sender.send(event).await;
        }

        panic!("kill everything")
    });
}
