use std::pin::Pin;

use crossterm::{
    event::{Event, EventStream, KeyCode},
    style::Color,
};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung::{soft, Renderer};

use oelung_lantern::{generate_sender, mpsc::Sender, PartialColumn, ReceiveEvent};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    let text = (0..100)
        .into_iter()
        .map(|line_num| format!("Line {line_num}"))
        .collect::<Vec<_>>();

    let mut current_top_line_num = 0;

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    render_screen(&mut renderer, &text, current_top_line_num)?;

    while let Some(world) = receiver.recv().await {
        let mut queued_effects: Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> = vec![];
        match world {
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('d') => {
                current_top_line_num += 1;
                render_screen(&mut renderer, &text, current_top_line_num)?;
            }
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('u') => {
                if current_top_line_num > 0 {
                    current_top_line_num -= 1;
                }
                render_screen(&mut renderer, &text, current_top_line_num)?;
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
    text: &[String],
    current_top_line_num: usize,
) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %PartialColumn::new(
              current_top_line_num,
              |line_num| soft! { %Text &text[line_num] }
          )
          %Text "(hit q to quit, u to scroll back, d to scroll down)"
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
