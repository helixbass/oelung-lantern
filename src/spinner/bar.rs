use std::time::Duration;

use crossterm::style::Color;
use oelung::{anyhow, Component, ComponentInterface, Grid, TextBuilder};
use tokio::{task::JoinHandle, time::interval};
use uuid::Uuid;

use crate::{mpsc::Sender, ReceiveEvent};

pub struct BarSpinner {
    pub color: Option<Color>,
    pub join_handle: JoinHandle<()>,
    pub next_step: u32,
    pub uuid: Uuid,
}

impl BarSpinner {
    pub fn new(period: Duration, color: Option<Color>, sender: Box<dyn Sender<Tick>>) -> Self {
        let uuid = Uuid::new_v4();
        let join_handle = tokio::spawn(async move {
            let mut interval = interval(period / 10);
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

impl Drop for BarSpinner {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

impl<'a> ComponentInterface for &'a BarSpinner {
    fn render<'b>(&self, _grid: Grid) -> Result<Component<'b>, anyhow::Error> {
        Ok({
            let mut text = TextBuilder::default();
            if let Some(color) = self.color {
                text = text.color(color);
            }
            let text = text.text_child(match self.next_step % 4 {
                0 => "|",
                1 => "/",
                2 => "-",
                3 => "\\",
                _ => unreachable!(),
            });
            text.build().unwrap().into()
        })
    }
}

impl ReceiveEvent<Tick> for BarSpinner {
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
