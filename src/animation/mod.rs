use std::pin::Pin;
use std::time::{Duration, Instant};

use derive_builder::Builder;
use futures::future::FutureExt;
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
    fn interpolate(&self, other: &Self, progress: f32) -> Self;
}

pub enum AnimationInstance {
    Running(AnimationInstanceRunning),
    Done { uuid: Uuid, repeat: AnimationRepeat },
}

impl AnimationInstance {
    pub fn new(
        repeat: AnimationRepeat,
        animation: Animation,
        sender: Box<dyn Sender<Event>>,
    ) -> Self {
        Self::Running(AnimationInstanceRunning::new(repeat, animation, sender))
    }

    pub fn current_progress(&self) -> f32 {
        let progress = self.current_progress_uneased();
        match self {
            Self::Done { .. } => progress,
            Self::Running(running) => running.animation.easing.interpolate(progress),
        }
    }

    pub fn current_progress_uneased(&self) -> f32 {
        match self {
            Self::Done { repeat, .. } => match repeat {
                AnimationRepeat::ForwardOnce => 1.0,
                AnimationRepeat::ForwardNTimes(_) => 1.0,
                AnimationRepeat::ForwardAndBackOnce => 0.0,
                AnimationRepeat::ForwardAndBackNTimes(_) => 1.0,
                _ => unreachable!(),
            },
            Self::Running(running) => {
                if running.is_done() {
                    return match running.repeat {
                        AnimationRepeat::ForwardOnce => 1.0,
                        AnimationRepeat::ForwardNTimes(_) => 1.0,
                        AnimationRepeat::ForwardAndBackOnce => 0.0,
                        AnimationRepeat::ForwardAndBackNTimes(_) => 1.0,
                        _ => unreachable!(),
                    };
                }
                let elapsed = running.started_at.elapsed();
                let num_durations =
                    (elapsed.as_millis() as f32) / (running.animation.duration.as_millis() as f32);
                let num_completed_durations = num_durations.floor();
                let progress_in_this_duration = num_durations - num_completed_durations;
                match running.repeat {
                    AnimationRepeat::ForwardOnce => {
                        assert!(num_completed_durations == 0.0);
                        progress_in_this_duration
                    }
                    AnimationRepeat::ForwardInfinite | AnimationRepeat::ForwardNTimes(_) => {
                        progress_in_this_duration
                    }
                    AnimationRepeat::ForwardAndBackOnce
                    | AnimationRepeat::ForwardAndBackInfinite
                    | AnimationRepeat::ForwardAndBackNTimes(_) => {
                        let is_going_forward = (num_completed_durations as u32) % 2 == 0;
                        if is_going_forward {
                            progress_in_this_duration
                        } else {
                            1.0 - progress_in_this_duration
                        }
                    }
                }
            }
        }
    }
}

impl ReceiveEvent<Tick> for AnimationInstance {
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        tick: &Tick,
        mut queue_effect: TQueueEffect,
    ) {
        let Self::Running(running) = self else {
            return;
        };
        if tick.uuid != running.uuid {
            return;
        }
        if running.is_done() {
            queue_effect({
                let sender = running.sender.box_clone();
                let uuid = running.uuid;
                async move {
                    sender.send(Done { uuid }.into()).await;
                }
                .boxed()
            });
            *self = AnimationInstance::Done {
                uuid: running.uuid,
                repeat: running.repeat,
            };
        }
    }
}

pub struct AnimationInstanceRunning {
    pub repeat: AnimationRepeat,
    pub animation: Animation,
    pub started_at: Instant,
    pub join_handle: JoinHandle<()>,
    pub uuid: Uuid,
    pub sender: Box<dyn Sender<Event>>,
}

impl AnimationInstanceRunning {
    pub fn new(
        repeat: AnimationRepeat,
        animation: Animation,
        sender: Box<dyn Sender<Event>>,
    ) -> Self {
        let uuid = Uuid::new_v4();
        let join_handle = tokio::spawn({
            let sender = sender.box_clone();
            async move {
                let mut interval = interval(Duration::from_millis(25));
                loop {
                    let _ = interval.tick().await;
                    sender.send(Tick { uuid }.into()).await;
                }
            }
        });

        Self {
            repeat,
            animation,
            started_at: Instant::now(),
            join_handle,
            uuid,
            sender,
        }
    }

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

impl Drop for AnimationInstanceRunning {
    fn drop(&mut self) {
        self.join_handle.abort();
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
