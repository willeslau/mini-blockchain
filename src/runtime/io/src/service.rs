use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use mio::Poll;
use crate::error::Error;

/// Dispatch and manages the IO handlers
pub struct IOService {

}

impl IOService {

}

struct IOServiceInner {
    thread: Option<JoinHandle<()>>,
    poll: Arc<Mutex<Poll>>,
}

impl IOServiceInner {
    pub fn start() -> Result<Self, Error> {
        let mut poll = Arc::new(Mutex::new(Poll::new()?));

        let io = IOServiceInner { thread: None, poll };

        std::thread::spawn(move || {

        });
        // Create a poll instance.

        // Create storage for events.
        let mut events = Events::with_capacity(128);

        // Setup the server socket.
        let addr = "127.0.0.1:13265".parse()?;
        let mut server = TcpListener::bind(addr)?;
        // Start listening for incoming connections.
        poll.registry()
            .register(&mut server, SERVER, Interest::READABLE)?;

        // Setup the client socket.
        let mut client = TcpStream::connect(addr)?;
        // Register the socket.
        poll.registry()
            .register(&mut client, CLIENT, Interest::READABLE | Interest::WRITABLE)?;

        // Start an event loop.
        loop {
            // Poll Mio for events, blocking until we get an event.
            poll.poll(&mut events, None)?;

            // Process each event.
            for event in events.iter() {
                // We can use the token we previously provided to `register` to
                // determine for which socket the event is.
                match event.token() {
                    SERVER => {
                        // If this is an event for the server, it means a connection
                        // is ready to be accepted.
                        //
                        // Accept the connection and drop it immediately. This will
                        // close the socket and notify the client of the EOF.
                        let connection = server.accept();
                        drop(connection);
                    }
                    CLIENT => {
                        if event.is_writable() {
                            // We can (likely) write to the socket without blocking.
                        }

                        if event.is_readable() {
                            // We can (likely) read from the socket without blocking.
                        }

                        // Since the server just shuts down the connection, let's
                        // just exit from our event loop.
                        return Ok(());
                    }
                    // We don't expect any events with tokens other than those we provided.
                    _ => unreachable!(),
                }
            }
        }
    }
}