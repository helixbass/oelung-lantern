use std::pin::Pin;

use crossterm::{
    event::{Event, EventStream, KeyCode},
    style::Color,
};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung::{soft, Renderer};

use oelung_lantern::{
    generate_sender, mpsc::Sender, spinner::snake, LoadingMessage, ReceiveEvent, SnakeSpinner,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let mut loading_message = LoadingMessage::new(
        SnakeSpinner::new(
            None,
            Some(Color::Red),
            Box::new(SnakeSpinnerTickSender::from(sender)),
        ),
        "Loading...".into(),
    );

    render_screen(&mut renderer, &loading_message)?;

    while let Some(world) = receiver.recv().await {
        let mut queued_effects: Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> = vec![];
        match world {
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            World::SnakeSpinnerTick(tick) => {
                loading_message.receive(&tick, |future| queued_effects.push(future))?;
                render_screen(&mut renderer, &loading_message)?;
            }
            _ => {}
        }
        for effect in queued_effects {
            tokio::spawn(effect);
        }
    }

    Ok(())
}

fn render_screen(
    renderer: &mut Renderer,
    loading_message: &LoadingMessage,
) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %loading_message
          %Text "(hit q to quit)"
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
