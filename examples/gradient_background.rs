use crossterm::{
    event::{Event, EventStream, KeyCode},
    style::Color,
};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung::{soft, Renderer};

use oelung_lantern::{generate_sender, mpsc::Sender, GradientBackground, GradientBuilder};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let text = "Example text";

    render_screen(&mut renderer, text)?;

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

fn render_screen(renderer: &mut Renderer, text: &str) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %GradientBackground::new(
            GradientBuilder::default()
                .start_color(Color::Rgb {
                    r: 20,
                    g: 20,
                    b: 45,
                })
                .end_color(Color::Rgb {
                    r: 20,
                    g: 20,
                    b: 245,
                }),
            soft! {
                %Text
                  text => text
            },
            16,
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
