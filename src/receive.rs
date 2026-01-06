use std::pin::Pin;

use oelung::anyhow;

pub trait ReceiveEvent<TEvent> {
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &TEvent,
        _queue_effect: TQueueEffect,
    ) -> Result<(), anyhow::Error>;
}
