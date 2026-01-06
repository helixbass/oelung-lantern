use std::pin::Pin;

use crossterm::event::{Event, EventStream, KeyCode};
use oelung::{soft, Renderer};
use smol_str::SmolStr;
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung_lantern::{generate_sender, mpsc::Sender, text_input, ReceiveEvent, TextInput};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let mut state = State::Inputting(TextInput::new(Box::new(TextInputDoneSender::from(sender))));

    render_screen(&mut renderer, &state)?;

    while let Some(world) = receiver.recv().await {
        let mut queued_effects: Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> = vec![];
        match world {
            // TODO: allow `q` in the text input
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            World::Crossterm(event) => {
                if let State::Inputting(text_input) = &mut state {
                    text_input.receive(&event, |future| queued_effects.push(future))?;
                }
                render_screen(&mut renderer, &state)?;
            }
            World::TextInputDone(done) => {
                state = State::Done(done.0);
                render_screen(&mut renderer, &state)?;
            }
        }
        for effect in queued_effects {
            tokio::spawn(effect);
        }
    }

    Ok(())
}

fn render_screen(renderer: &mut Renderer, state: &State) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          match state {
              State::Inputting(text_input) => soft! { %text_input },
              State::Done(value) => soft! {
                %Text children => [
                  %Text "You entered: "
                  %Text value
                ]
              },
          }
          %Text "(hit q to quit)"
        ]
    })?;

    Ok(())
}

enum World {
    Crossterm(Event),
    TextInputDone(text_input::Done),
}

generate_sender!(World, Crossterm, Event);
generate_sender!(World, TextInputDone, text_input::Done);

enum State {
    Inputting(TextInput),
    Done(SmolStr),
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
