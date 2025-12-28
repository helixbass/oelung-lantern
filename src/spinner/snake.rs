use std::time::Duration;

use crossterm::style::Color;
use oelung::{anyhow, Component, ComponentInterface, Grid};
use tokio::{task::JoinHandle, time::interval};
use uuid::Uuid;

use crate::{mpsc::Sender, ReceiveEvent};

pub struct SnakeSpinner {
    pub color: Option<Color>,
    pub join_handle: JoinHandle<()>,
    pub next_step: u32,
    pub uuid: Uuid,
}

impl SnakeSpinner {
    pub fn new(period: Duration, color: Option<Color>, sender: Box<dyn Sender<Tick>>) -> Self {
        let uuid = Uuid::new_v4();
        let join_handle = tokio::spawn(async move {
            let mut interval = interval(period);
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
    fn render<'b>(&self, _grid: Grid) -> Result<Component<'b>, anyhow::Error> {
        unimplemented!()
    }
}

impl ReceiveEvent<Tick> for SnakeSpinner {
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
