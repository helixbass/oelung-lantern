use std::pin::Pin;

use oelung::anyhow;

pub trait ReceiveEvent<TEvent, TReturn = ()> {
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &TEvent,
        _queue_effect: TQueueEffect,
    ) -> Result<TReturn, anyhow::Error>;
}
