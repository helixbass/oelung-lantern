use std::pin::Pin;
use std::time::Duration;

use crossterm::{
    event::{Event, EventStream, KeyCode},
    style::Color,
};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;
// use tracing_chrome::ChromeLayerBuilder;
// use tracing_subscriber::prelude::*;

use oelung::{soft, Renderer};

use oelung_lantern::{
    animated_gradient, generate_sender, mpsc::Sender, AnimatedGradient, AnimatedGradientBuilder,
    AnimationBuilder, AnimationRepeat, Easing, ReceiveEvent,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
    // tracing_subscriber::registry().with(chrome_layer).init();

    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let mut gradient = AnimatedGradientBuilder::default()
        .start_color(Color::Rgb {
            r: 20,
            g: 20,
            b: 45,
        })
        .end_color(Color::Rgb {
            r: 20,
            g: 20,
            b: 245,
        })
        .finish_start_color(Color::Rgb {
            r: 245,
            g: 20,
            b: 20,
        })
        .finish_end_color(Color::Rgb {
            r: 45,
            g: 20,
            b: 20,
        })
        .height(10)
        .width(80)
        .animation_repeat(AnimationRepeat::ForwardAndBackInfinite)
        .animation(
            AnimationBuilder::default()
                .duration(Duration::from_millis(2400))
                .easing(Easing::Linear)
                .build()
                .unwrap(),
        )
        .sender(Box::new(AnimatedGradientSender::from(sender)))
        .build()
        .unwrap();

    render_screen(&mut renderer, &gradient)?;

    while let Some(world) = receiver.recv().await {
        let mut queued_effects: Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> = vec![];
        match world {
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            World::AnimatedGradient(animated_gradient::Event::Tick(tick)) => {
                gradient.receive(&tick, |future| queued_effects.push(future));
                render_screen(&mut renderer, &gradient)?;
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
    gradient: &AnimatedGradient,
) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %gradient
          %Text "(hit q to quit)"
        ]
    })?;

    Ok(())
}

enum World {
    Crossterm(Event),
    AnimatedGradient(animated_gradient::Event),
}

generate_sender!(World, Crossterm, Event);
generate_sender!(World, AnimatedGradient, animated_gradient::Event);

fn listen_to_crossterm_events(sender: CrosstermSender) {
    tokio::spawn(async move {
        let mut event_stream = EventStream::new();

        while let Some(Ok(event)) = event_stream.next().await {
            sender.send(event).await;
        }

        panic!("kill everything")
    });
}
