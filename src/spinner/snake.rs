use std::time::Duration;

use crossterm::style::Color;
use tokio::{sync::mpsc, task::JoinHandle, time::interval};

use crate::SendRerender;

pub struct SnakeSpinner {
    pub color: Option<Color>,
    pub join_handle: JoinHandle<()>,
    pub sender: Box<dyn SendRerender>,
    pub internal_receiver: mpsc::Receiver<()>,
    pub next_step: u32,
}

impl SnakeSpinner {
    pub fn new(period: Duration, color: Option<Color>, sender: Box<dyn SendRerender>) -> Self {
        let (internal_sender, internal_receiver) = mpsc::channel::<()>(100);
        let join_handle = tokio::spawn(async move {
            let interval = interval(period);
            loop {
                let _ = interval.tick().await;
                internal_sender.send(()).unwrap();
            }
        });

        Self {
            color,
            join_handle,
            sender,
            internal_receiver,
            next_step: 0,
        }
    }
}

impl Drop for SnakeSpinner {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}
