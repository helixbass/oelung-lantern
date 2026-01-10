use std::pin::Pin;
use std::sync::LazyLock;
use std::time::Duration;

use crossterm::style::Color;
use oelung::{anyhow, Component, ComponentInterface, Grid, TextBuilder};
use tokio::{task::JoinHandle, time::interval};
use tracing::instrument;
use uuid::Uuid;

use crate::{mpsc::Sender, ReceiveEvent};

pub const DEFAULT_PERIOD: u64 = 1000;

fn steps() -> &'static [char] {
    static STEPS: LazyLock<Vec<char>> =
        LazyLock::new(|| vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']);
    &*STEPS
}

pub struct SnakeSpinner {
    pub color: Option<Color>,
    pub join_handle: JoinHandle<()>,
    pub next_step: usize,
    pub uuid: Uuid,
}

impl SnakeSpinner {
    #[instrument(level = "trace", skip(period, color, sender))]
    pub fn new(
        period: Option<Duration>,
        color: Option<Color>,
        sender: Box<dyn Sender<Tick>>,
    ) -> Self {
        let period = period.unwrap_or_else(|| Duration::from_millis(DEFAULT_PERIOD));
        let uuid = Uuid::new_v4();
        let join_handle = tokio::spawn(async move {
            let mut interval = interval(period / u32::try_from(steps().len()).unwrap());
            loop {
                let _ = interval.tick().await;
                sender.send(Tick { uuid }).await;
            }
        });

        Self {
            color,
            join_handle,
            next_step: 0,
            uuid,
        }
    }
}

impl Drop for SnakeSpinner {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

impl<'a> ComponentInterface for &'a SnakeSpinner {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render(&self, _grid: Grid) -> Result<Component<'_>, anyhow::Error> {
        // TODO: maybe expose `maybe_color => self.color`
        // on `%Text`?
        // Ok(soft! {
        //   %Text
        //     color =>
        // })
        Ok({
            let mut text = TextBuilder::default();
            if let Some(color) = self.color {
                text = text.color(color);
            }
            let text = text.text_child(steps()[self.next_step % steps().len()]);
            text.build().unwrap().into()
        })
    }
}

impl ReceiveEvent<Tick> for SnakeSpinner {
    #[instrument(level = "trace", skip(self, event, _queue_effect))]
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &Tick,
        _queue_effect: TQueueEffect,
    ) -> Result<(), anyhow::Error> {
        if event.uuid != self.uuid {
            return Ok(());
        }
        self.next_step += 1;

        Ok(())
    }
}

pub struct Tick {
    pub uuid: Uuid,
}
