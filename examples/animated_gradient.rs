use crossterm::{
    event::{Event, EventStream, KeyCode},
    style::Color,
};
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;

use oelung::{soft, Renderer};

use oelung_lantern::{
    animated_gradient, generate_sender, mpsc::Sender, AnimatedGradient, AnimationBuilder,
    AnimationRepeat,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut renderer = Renderer::try_new()?;

    let (sender, mut receiver) = channel::<World>(100);

    listen_to_crossterm_events(CrosstermSender::from(sender.clone()));

    let gradient = AnimatedGradient::new(
        Color::Rgb {
            r: 20,
            g: 20,
            b: 45,
        },
        Color::Rgb {
            r: 20,
            g: 20,
            b: 245,
        },
        Color::Rgb {
            r: 45,
            g: 20,
            b: 20,
        },
        Color::Rgb {
            r: 245,
            g: 20,
            b: 20,
        },
        1,
        40,
        AnimationRepeat::ForwardAndBackInfinite,
        AnimationBuilder::default()
            .duration(Duration::from_millis(2400))
            .easing(Easing::Linear)
            .build()
            .unwrap(),
        Box::new(AnimatedGradientTickSender::from(sender)),
    );

    render_screen(&mut renderer, &gradient)?;

    while let Some(world) = receiver.recv().await {
        let mut queued_effects: Vec<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> = vec![];
        match world {
            World::Crossterm(Event::Key(key)) if key.code == KeyCode::Char('q') => {
                break;
            }
            World::AnimatedGradientTick(tick) => {
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
    AnimatedGradientTick(animated_gradient::Tick),
}

generate_sender!(World, Crossterm, Event);
generate_sender!(World, AnimatedGradientTick, animated_gradient::Tick);

fn listen_to_crossterm_events(sender: CrosstermSender) {
    tokio::spawn(async move {
        let mut event_stream = EventStream::new();

        while let Some(Ok(event)) = event_stream.next().await {
            sender.send(event).await;
        }

        panic!("kill everything")
    });
}
