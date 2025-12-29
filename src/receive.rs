use std::pin::Pin;

pub trait ReceiveEvent<TEvent> {
    fn receive<TQueueEffect: FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>)>(
        &mut self,
        event: &TEvent,
        _queue_effect: TQueueEffect,
    );
}
