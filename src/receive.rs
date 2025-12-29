pub trait ReceiveEvent<TEvent> {
    fn receive<TQueueEffect: FnMut(Box<dyn Future<Output = ()>>)>(
        &mut self,
        event: &TEvent,
        _queue_effect: TQueueEffect,
    );
}
