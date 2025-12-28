pub trait ReceiveEvent<TEvent> {
    fn receive(&mut self, event: &TEvent);
}
