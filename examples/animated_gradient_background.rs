use std::pin::Pin;
use std::time::Duration;

use crossterm::{
    event::{Event, EventStream, KeyCode},
    style::Color,
};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung::{soft, Renderer};

use oelung_lantern::{
    animation, generate_sender, mpsc::Sender, AnimatedGradientBackground, AnimatedGradientBuilder,
    AnimationBuilder, AnimationInstance, AnimationRepeat, Easing, ReceiveEvent,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let text = "Example text";

    let mut animation = AnimationInstance::new(
        AnimationRepeat::ForwardAndBackInfinite,
        AnimationBuilder::default()
            .duration(Duration::from_millis(1500))
            .easing(Easing::Linear)
            .build()
            .unwrap(),
        Box::new(AnimationSender::from(sender.clone())),
    );

    render_screen(&mut renderer, text, &animation)?;

    while let Some(world) = receiver.recv().await {
        let mut queued_effects: Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> = vec![];
        match world {
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            World::Animation(animation::Event::Tick(tick)) => {
                animation.receive(&tick, |future| queued_effects.push(future))?;
                render_screen(&mut renderer, text, &animation)?;
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
    text: &str,
    animation: &AnimationInstance,
) -> Result<(), anyhow::Error> {
    renderer.render(soft! {
      %FlexColumn
        children => [
          %AnimatedGradientBackground::new(
            AnimatedGradientBuilder::default()
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
                    r: 20,
                    g: 240,
                    b: 20,
                })
                .finish_end_color(Color::Rgb {
                    r: 20,
                    g: 70,
                    b: 20,
                })
                .animation_instance(animation),
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
    Animation(animation::Event),
}

generate_sender!(World, Crossterm, Event);
generate_sender!(World, Animation, animation::Event);

fn listen_to_crossterm_events(sender: CrosstermSender) {
    tokio::spawn(async move {
        let mut event_stream = EventStream::new();

        while let Some(Ok(event)) = event_stream.next().await {
            sender.send(event).await;
        }

        panic!("kill everything")
    });
}
