use std::pin::Pin;

use oelung::{anyhow, soft, Component, ComponentInterface, Grid};
use smol_str::SmolStr;
use tracing::instrument;

use crate::{spinner::snake, ReceiveEvent, SnakeSpinner};

pub struct LoadingMessage {
    pub spinner: SnakeSpinner,
    pub message: SmolStr,
}

impl LoadingMessage {
    pub fn new(spinner: SnakeSpinner, message: SmolStr) -> Self {
        Self { spinner, message }
    }
}

impl<'a> ComponentInterface for &'a LoadingMessage {
    #[instrument(level = "trace", skip(self, _grid))]
    fn render<'b: 'c, 'c>(&'c self, _grid: Grid) -> Result<Component<'b>, anyhow::Error> {
        Ok(soft! {
            %Text children => [
              %&self.spinner
              %Text " "
              %Text &self.message
            ]
        })
    }

    fn height(&self) -> Option<u16> {
        Some(1)
    }
}

impl ReceiveEvent<snake::Tick> for LoadingMessage {
    #[instrument(level = "trace", skip(self, event, queue_effect))]
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &snake::Tick,
        queue_effect: TQueueEffect,
    ) {
        self.spinner.receive(event, queue_effect);
    }
}
