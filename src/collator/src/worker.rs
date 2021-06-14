use std::sync::mpsc::{Receiver, Sender};
use crate::traits::Collator as CollatorTrait;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool};
use std::time::{Duration, Instant};
use std::thread;
use std::thread::JoinHandle;

pub enum Message<Executable: transaction::Executable> {
    Job(Executable),
    Executable(Vec<Executable>),
    Terminate,
}

pub struct CollatorWorker<Executable, Collator>
    where
        Executable: transaction::Executable,
        Collator: CollatorTrait,
{
    /// The target block interval, in seconds
    block_target_time: u8,
    block_size: usize,
    executable_rx: Arc<Mutex<Receiver<Message<Executable>>>>,
    /// The channel to send the executables for block production
    block_tx: Sender<Message<Executable>>,
    collator: Collator,

    // internal states
    started: AtomicBool,
}

impl <Executable, Collator> CollatorWorker<Executable, Collator>
    where
        Executable: transaction::Executable + Send + 'static,
        Collator: CollatorTrait<Executable = Executable> + Send + Sync + 'static,
{
    pub fn new(
        block_target_time: u8,
        block_size: usize,
        collator: Collator,
        executable_rx: Arc<Mutex<Receiver<Message<Executable>>>>,
        block_tx: Sender<Message<Executable>>,
    ) -> Self {
        CollatorWorker {
            block_target_time,
            block_size,
            executable_rx,
            block_tx,
            collator,
            started: AtomicBool::new(false),
        }
    }

    /// Start the collator. This would trigger a new thread to run.
    /// Implement stop according to https://doc.rust-lang.org/book/ch20-03-graceful-shutdown-and-cleanup.html
    pub fn start(&mut self) -> JoinHandle<()> {
        // if self.started.into_inner() { return; }
        // self.started.compare_exchange(false,true,Ordering::SeqCst,Ordering::Acquire);

        let rx = self.executable_rx.clone();
        let tx = self.block_tx.clone();
        let block_size = self.block_size;
        let collator = Arc::new(Mutex::new(self.collator.clone()));
        let block_target = Duration::new(0, (self.block_target_time as u32) * 1000_000 );

        thread::spawn(move || {
            let mut last_updated_time = Instant::now();

            // Only one thread will have access to this.
            let rx = rx.lock().unwrap();
            let mut collator = collator.lock().unwrap();

            loop {
                let r = rx.recv_timeout(block_target);
                if r.is_ok() {
                    let m = r.unwrap();
                    match m {
                        Message::Terminate => {
                            tx.send(Message::Terminate);
                            break;
                        }
                        Message::Job(e) => { collator.add_executable(e); }
                        _ => {}
                    }
                }

                let elapsed = last_updated_time.elapsed();
                if collator.size() == block_size || elapsed.gt(&block_target) {
                    let executables = collator.dump();
                    collator.clear();
                    tx.send(Message::Executable(executables));
                    last_updated_time = Instant::now();
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::default::DefaultCollator;
    use crate::worker::{CollatorWorker, Message};
    use transaction::MockedExecutable;
    use std::sync::mpsc::{channel};
    use std::thread;
    use std::sync::{Arc, Mutex};

    #[test]
    fn collator_works() {
        let (block_tx, block_rx) = channel();
        let (exe_tx, exe_rx) = channel();

        let collator = DefaultCollator::new();
        let mut worker = CollatorWorker::new(
            2,
            10,
            collator,
            Arc::new(Mutex::new(exe_rx)),
            block_tx,
        );

        let mut threads = vec![];
        for _ in 0..10 {
            let tx = exe_tx.clone();
            threads.push(thread::spawn(move || {
                for i in 0..1000 {
                    let message = Message::Job(MockedExecutable::new(i.to_string()));
                    tx.send(message);
                }
            }));
        }

        let block_rx = Arc::new(Mutex::new(block_rx));
        thread::spawn(move || {
            let mut count: usize = 0;
            loop {
                let x = block_rx.lock().unwrap().recv().unwrap();
                match x {
                    Message::Terminate => {
                        break;
                    },
                    Message::Executable(v) => { count += v.len(); }
                    _ => { }
                }
            }
            assert_eq!(count, 10000);
        });

        let h = worker.start();

        for t in threads {
            t.join();
        }

        exe_tx.send(Message::Terminate);
        h.join();
    }
}