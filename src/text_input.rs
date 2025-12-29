use std::pin::Pin;

use crossterm::event::{Event, KeyCode};
use futures::future::FutureExt;
use oelung::{anyhow, soft, Component, ComponentInterface, Grid};
use smol_str::{SmolStr, ToSmolStr};
use squalid::_d;

use crate::{mpsc::Sender, ReceiveEvent};

pub struct TextInput {
    pub input: String,
    pub sender: Box<dyn Sender<Done>>,
}

impl TextInput {
    pub fn new(sender: Box<dyn Sender<Done>>) -> Self {
        Self {
            input: _d(),
            sender,
        }
    }
}

impl<'a> ComponentInterface for &'a TextInput {
    fn render<'b>(&self, _grid: Grid) -> Result<Component<'b>, anyhow::Error> {
        Ok(soft! {
            %Text &self.input
        })
    }

    fn height(&self) -> Option<u16> {
        Some(1)
    }
}

impl ReceiveEvent<Event> for TextInput {
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &Event,
        mut queue_effect: TQueueEffect,
    ) {
        match event {
            Event::Key(key) if matches!(key.code, KeyCode::Char(_)) => {
                let ch = match key.code {
                    KeyCode::Char(ch) => ch,
                    _ => panic!("unreachable"),
                };
                self.input.push(ch);
            }
            Event::Key(key) if key.code == KeyCode::Enter => {
                queue_effect({
                    let sender = self.sender.box_clone();
                    let input = self.input.to_smolstr();
                    async move {
                        sender.send(Done(input)).await;
                    }
                    .boxed()
                });
            }
            _ => {}
        }
    }
}

pub struct Done(pub SmolStr);
