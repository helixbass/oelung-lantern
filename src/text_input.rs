use crossterm::event::{Event, KeyCode};
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
}

impl ReceiveEvent<Event> for TextInput {
    fn receive(&mut self, event: &Event) {
        match event {
            Event::Key(key) if matches!(key.code, KeyCode::Char(_)) => {
                let ch = match key.code {
                    KeyCode::Char(ch) => ch,
                    _ => panic!("unreachable"),
                };
                self.input.push(ch);
            }
            Event::Key(key) if key.code == KeyCode::Enter => {
                self.sender.send(Done(self.input.to_smolstr())).await;
            }
            _ => {}
        }
    }
}

pub struct Done(SmolStr);
