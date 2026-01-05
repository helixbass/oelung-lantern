use crossterm::event::{Event, EventStream, KeyCode};
use oelung::{soft, Renderer};
use smol_str::ToSmolStr;
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung_lantern::{generate_sender, mpsc::Sender, tabs, Tabs};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let mut selected_tab = Tab::Home;

    render_screen(&mut renderer, selected_tab)?;

    while let Some(world) = receiver.recv().await {
        match world {
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('h') => {
                selected_tab = Tab::Home;
                render_screen(&mut renderer, selected_tab)?;
            }
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('b') => {
                selected_tab = Tab::Blog;
                render_screen(&mut renderer, selected_tab)?;
            }
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('c') => {
                selected_tab = Tab::Contact;
                render_screen(&mut renderer, selected_tab)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn render_screen(renderer: &mut Renderer, selected_tab: Tab) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %Tabs::new(
            [
                tabs::Tab::new(
                    "Home".to_smolstr(),
                    soft! {
                      %Text "Home content"
                    }
                ),
                tabs::Tab::new(
                    "Blog".to_smolstr(),
                    soft! {
                      %Text "Blog content"
                    }
                ),
                tabs::Tab::new(
                    "Contact".to_smolstr(),
                    soft! {
                      %Text "Contact content"
                    }
                ),
            ],
            match selected_tab {
                Tab::Home => 0,
                Tab::Blog => 1,
                Tab::Contact => 2,
            },
            Some(1.0),
          )
          %Text "(hit h for Home, b for Blog, c for Contact, q to quit)"
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Home,
    Blog,
    Contact,
}
