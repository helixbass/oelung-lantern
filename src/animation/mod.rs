use std::time::{Duration, Instant};

use derive_builder::Builder;
use tokio::{task::JoinHandle, time::interval};
use uuid::Uuid;

use crate::mpsc::Sender;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AnimationRepeat {
    ForwardOnce,
    ForwardInfinite,
    ForwardNTimes(u32),
    ForwardAndBackOnce,
    ForwardAndBackInfinite,
    ForwardAndBackNTimes(u32),
}

#[derive(Builder, Clone)]
pub struct Animation {
    pub duration: Duration,
    pub easing: Easing,
}

#[derive(Clone)]
pub enum Easing {
    Linear,
}

impl Easing {
    pub fn interpolate(&self, progress: f32) -> f32 {
        assert!(progress >= 0.0 && progress <= 1.0);
        match self {
            Self::Linear => progress,
        }
    }
}

pub trait Interpolateable {
    fn interpolate(start: &Self, end: &Self, progress: f32) -> Self;
}

pub enum AnimationInstance {
    Running {
        repeat: AnimationRepeat,
        animation: Animation,
        started_at: Instant,
        join_handle: JoinHandle<()>,
        uuid: Uuid,
    },
    Done,
}

impl AnimationInstance {
    pub fn new(
        repeat: AnimationRepeat,
        animation: Animation,
        sender: Box<dyn Sender<Event>>,
    ) -> Self {
        let uuid = Uuid::new_v4();
        let join_handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(25));
            loop {
                let _ = interval.tick().await;
                sender.send(Tick { uuid }.into()).await;
            }
        });

        Self::Running {
            repeat,
            animation,
            started_at: Instant::now(),
            join_handle,
            uuid,
        }
    }
}

impl ReceiveEvent<Event> for AnimationInstance {
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &Event,
        _queue_effect: TQueueEffect,
    ) {
        if event.uuid() != self.uuid {
            return;
        }
        self.next_step += 1;
    }
}

pub struct Tick {
    pub uuid: Uuid,
}

pub struct Done {
    pub uuid: Uuid,
}

pub enum Event {
    Tick(Tick),
    Done(Done),
}

impl Event {
    pub fn uuid(&self) -> Uuid {
        match self {
            Self::Tick(tick) => tick.uuid,
            Self::Done(done) => done.uuid,
        }
    }
}

impl From<Tick> for Event {
    fn from(value: Tick) -> Self {
        Self::Tick(value)
    }
}

impl From<Done> for Event {
    fn from(value: Done) -> Self {
        Self::Done(value)
    }
}
