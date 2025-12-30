use std::pin::Pin;
use std::time::{Duration, Instant};

use derive_builder::Builder;
use tokio::{task::JoinHandle, time::interval};
use uuid::Uuid;

use crate::{mpsc::Sender, ReceiveEvent};

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
    Running(AnimationInstanceRunning),
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

        Self::Running(AnimationInstanceRunning {
            repeat,
            animation,
            started_at: Instant::now(),
            join_handle,
            uuid,
        })
    }
}

impl ReceiveEvent<Tick> for AnimationInstance {
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        tick: &Tick,
        _queue_effect: TQueueEffect,
    ) {
        let Self::Running(running) = self else {
            return;
        };
        if tick.uuid != running.uuid {
            return;
        }
        if running.is_done() {
            *self = AnimationInstance::Done;
        }
    }
}

pub struct AnimationInstanceRunning {
    pub repeat: AnimationRepeat,
    pub animation: Animation,
    pub started_at: Instant,
    pub join_handle: JoinHandle<()>,
    pub uuid: Uuid,
}

impl AnimationInstanceRunning {
    pub fn is_done(&self) -> bool {
        let elapsed = self.started_at.elapsed();
        match self.repeat {
            AnimationRepeat::ForwardOnce => elapsed > self.animation.duration,
            AnimationRepeat::ForwardInfinite => false,
            AnimationRepeat::ForwardNTimes(n) => elapsed > self.animation.duration * n,
            AnimationRepeat::ForwardAndBackOnce => elapsed > self.animation.duration * 2,
            AnimationRepeat::ForwardAndBackInfinite => false,
            AnimationRepeat::ForwardAndBackNTimes(n) => elapsed > self.animation.duration * n * 2,
        }
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
