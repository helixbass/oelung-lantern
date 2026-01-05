use crossterm::event::{Event, EventStream, KeyCode};
use oelung::{soft, Renderer};
use smol_str::ToSmolStr;
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung_lantern::{generate_sender, mpsc::Sender, Tab, Tabs};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    render_screen(&mut renderer)?;

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

fn render_screen(renderer: &mut Renderer) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %Tabs::new(
            [
                Tab::new(
                    "Home".to_smolstr(),
                    soft! {
                      %Text "Home content"
                    }
                ),
                Tab::new(
                    "Blog".to_smolstr(),
                    soft! {
                      %Text "Blog content"
                    }
                ),
                Tab::new(
                    "Contact".to_smolstr(),
                    soft! {
                      %Text "Contact content"
                    }
                ),
            ],
            0
          )
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
