use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread::JoinHandle;
use std::time::Duration;
use mio::{event, Events, Interest, Poll, Token};
use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use slab::Slab;
use crate::error::Error;
use crate::handler::IoHandler;
use common::ensure;

const MAX_TOKEN: usize = 1024;

/// Dispatch and manages the IO handlers
pub struct IOService {}

impl IOService {}

pub enum NetworkIOMessage<Message> {
    /// A message to handle for the event loop
    Message(Message),
}

struct IOServiceInner<Message> {
    is_stopped: AtomicBool,
    /// The work stealing deque to a pool of Worker threads
    worker_deque: crossbeam_deque::Worker<Message>,
    /// The event loop poll
    poll: Poll,
    handlers: HashMap<usize, Box<dyn IoHandler<Message>>>,
}

impl<Message> IOServiceInner<Message> {
    pub fn new() -> Result<Self, Error> {
        let w = crossbeam_deque::Worker::new_fifo();
        Ok(Self {
            is_stopped: AtomicBool::new(false),
            worker_deque: w,
            poll: Poll::new()?,
            handlers: Default::default(),
        })
    }

    /// Start an event loop.
    pub fn start(&mut self) {
        let mut events = Events::with_capacity(1024);
        loop {
            if self.is_stopped.load(Ordering::SeqCst) { break; }

            // Poll Mio for events, blocking until we get an event.
            self.poll.poll(&mut events, Some(Duration::from_millis(2000))).expect("cannot poll event");

            // Process each event.
            for event in events.iter() {
                self.dispatch_event(event);
            }
        }
    }

    pub fn dispatch_event(&mut self, event: &Event) {}

    pub fn register<S: event::Source + ?Sized>(
        &mut self,
        source: &mut S,
        token: Token,
        interest: Interest,
        handler: Box<dyn IoHandler<Message>>,
    ) -> Result<(), Error> {
        ensure!(token.0 <= MAX_TOKEN, Error::InvalidTokenSize)?;
        self.handlers.insert(token.0, handler);
        self.poll.registry().register(source, token, interest);
        Ok(())
    }

    pub fn deregister<S: event::Source + ?Sized>(
        &mut self,
        source: &mut S,
        token: Token,
    ) -> Result<(), Error> {
        ensure!(token.0 <= MAX_TOKEN, Error::InvalidTokenSize)?;
        self.handlers.remove(&token.0);
        self.poll.registry().deregister(source);
        Ok(())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn slab_works() {
        let mut s = slab::Slab::new();
        let i = s.insert(123);
        let j = s.insert(124);
        println!("{}, {}", i, j);
    }
}