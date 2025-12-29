use std::time::{Duration, Instant};

use oelung::{anyhow, soft, Component, ComponentInterface, Grid};
use tokio::{task::JoinHandle, time::interval};
use uuid::Uuid;

use crate::mpsc::Sender;

pub struct Countdown {
    pub total: Duration,
    pub started_at: Instant,
    pub join_handle: JoinHandle<()>,
    pub uuid: Uuid,
}

impl Countdown {
    pub fn new(total: Duration, sender: Box<dyn Sender<Tick>>) -> Self {
        let uuid = Uuid::new_v4();
        let join_handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(1));
            loop {
                let _ = interval.tick().await;
                sender.send(Tick { uuid }).await;
            }
        });

        Self {
            total,
            join_handle,
            started_at: Instant::now(),
            uuid,
        }
    }
}

impl Drop for Countdown {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

impl<'a> ComponentInterface for &'a Countdown {
    fn render<'b>(&self, _grid: Grid) -> Result<Component<'b>, anyhow::Error> {
        let remaining = if self.total <= self.started_at.elapsed() {
            "0".to_owned()
        } else {
            let remaining = self.total - self.started_at.elapsed();
            format!("{:.3}", (remaining.as_millis() as f64) / 1000.0)
        };
        Ok(soft! {
            %Text children => [
              %Text remaining
              %Text "s"
            ]
        })
    }

    fn height(&self) -> Option<u16> {
        Some(1)
    }
}

pub struct Tick {
    pub uuid: Uuid,
}
