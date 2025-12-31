use std::pin::Pin;
use std::sync::LazyLock;
use std::time::Duration;

use oelung::{anyhow, Component, ComponentInterface, Grid, TextBuilder};
use tokio::{task::JoinHandle, time::interval};
use tracing::instrument;
use uuid::Uuid;

use crate::{mpsc::Sender, ReceiveEvent};

fn steps() -> &'static [&'static str] {
    static STEPS: LazyLock<Vec<&'static str>> =
        LazyLock::new(|| vec!["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"]);
    &*STEPS
}

pub struct MoonSpinner {
    pub join_handle: JoinHandle<()>,
    pub next_step: usize,
    pub uuid: Uuid,
}

impl MoonSpinner {
    #[instrument(level = "trace", skip(period, sender))]
    pub fn new(period: Option<Duration>, sender: Box<dyn Sender<Tick>>) -> Self {
        let period = period.unwrap_or_else(|| Duration::from_millis(1600));
        let uuid = Uuid::new_v4();
        let join_handle = tokio::spawn(async move {
            let mut interval = interval(period / u32::try_from(steps().len()).unwrap());
            loop {
                let _ = interval.tick().await;
                sender.send(Tick { uuid }).await;
            }
        });

        Self {
            join_handle,
            next_step: 0,
            uuid,
        }
    }
}

impl Drop for MoonSpinner {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

impl<'a> ComponentInterface for &'a MoonSpinner {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render(&self, _grid: Grid) -> Result<Component<'_>, anyhow::Error> {
        Ok({
            let text = TextBuilder::default();
            let text = text.text_child(steps()[self.next_step % steps().len()]);
            text.build().unwrap().into()
        })
    }
}

impl ReceiveEvent<Tick> for MoonSpinner {
    #[instrument(level = "trace", skip(self, event, _queue_effect))]
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &Tick,
        _queue_effect: TQueueEffect,
    ) {
        if event.uuid != self.uuid {
            return;
        }
        self.next_step += 1;
    }
}

pub struct Tick {
    pub uuid: Uuid,
}
