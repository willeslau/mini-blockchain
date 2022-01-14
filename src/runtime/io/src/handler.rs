pub type HandlerId = usize;

/// The message struct to be passed around btw threads and workers
pub enum IOMessage<Message> where Message: Send + Sized {
    Tmp(Message)
}

/// Generic IO handler.
/// All the handler function are called from within IO event loop.
/// `Message` type is used as notification data
pub trait IoHandler<Message>: Send + Sync
where
    Message: Send + Sync + 'static,
{
    fn initialize(&self) {}
    // /// Initialize the handler
    // fn initialize(&self, _io: &IoContext<Message>) {}
    // /// Timer function called after a timeout created with `HandlerIo::timeout`.
    // fn timeout(&self, _io: &IoContext<Message>, _timer: TimerToken) {}
    // /// Called when a broadcasted message is received. The message can only be sent from a different IO handler.
    // fn message(&self, _io: &IoContext<Message>, _message: &Message) {}
    // /// Called when an IO stream gets closed
    // fn stream_hup(&self, _io: &IoContext<Message>, _stream: StreamToken) {}
    // /// Called when an IO stream can be read from
    // fn stream_readable(&self, _io: &IoContext<Message>, _stream: StreamToken) {}
    // /// Called when an IO stream can be written to
    // fn stream_writable(&self, _io: &IoContext<Message>, _stream: StreamToken) {}
    // /// Register a new stream with the event loop
    // fn register_stream(
    //     &self,
    //     _stream: StreamToken,
    //     _reg: Token,
    //     _event_loop: &mut EventLoop<IoManager<Message>>,
    // ) {
    // }
    // /// Re-register a stream with the event loop
    // fn update_stream(
    //     &self,
    //     _stream: StreamToken,
    //     _reg: Token,
    //     _event_loop: &mut EventLoop<IoManager<Message>>,
    // ) {
    // }
    // /// Deregister a stream. Called when stream is removed from event loop
    // fn deregister_stream(
    //     &self,
    //     _stream: StreamToken,
    //     _event_loop: &mut EventLoop<IoManager<Message>>,
    // ) {
    // }
}
