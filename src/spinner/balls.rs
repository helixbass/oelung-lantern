use std::sync::LazyLock;
use std::time::Duration;

use crossterm::style::Color;
use oelung::{anyhow, Component, ComponentInterface, Grid, TextBuilder};
use tokio::{task::JoinHandle, time::interval};
use uuid::Uuid;

use crate::{mpsc::Sender, ReceiveEvent};

fn steps() -> &'static [char] {
    static STEPS: LazyLock<Vec<char>> = LazyLock::new(|| vec!['⢄', '⢂', '⢁', '⡁', '⡈', '⡐', '⡠']);
    &*STEPS
}

pub struct BallsSpinner {
    pub color: Option<Color>,
    pub join_handle: JoinHandle<()>,
    pub next_step: usize,
    pub uuid: Uuid,
}

impl BallsSpinner {
    pub fn new(
        period: Option<Duration>,
        color: Option<Color>,
        sender: Box<dyn Sender<Tick>>,
    ) -> Self {
        let period = period.unwrap_or_else(|| Duration::from_millis(910));
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

impl Drop for BallsSpinner {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

impl<'a> ComponentInterface for &'a BallsSpinner {
    fn render<'b>(&self, _grid: Grid) -> Result<Component<'b>, anyhow::Error> {
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

impl ReceiveEvent<Tick> for BallsSpinner {
    fn receive(&mut self, event: &Tick) {
        if event.uuid != self.uuid {
            return;
        }
        self.next_step += 1;
    }
}

pub struct Tick {
    pub uuid: Uuid,
}
