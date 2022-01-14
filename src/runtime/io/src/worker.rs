use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;
use crossbeam_deque::Steal;
use crate::handler::{HandlerId, IoHandler};

const STACK_SIZE: usize = 16 * 1024 * 1024;

/// The type of work to do
pub enum WorkType<Message> {
    // Read,
    // Write,
    Message(Arc<Message>),
}

// pub struct Work<Message: Send + Sync + 'static, Handler: IoHandler<Message>> {
//     work_type: WorkType<Message>,
//     handler: Arc<Handler>,
//     handler_id: HandlerId,
// }

// NOTE: this would require Message to be Send + Sync + 'static
// pub struct Work<Message, Handler: IoHandler<Message>> {
//     work_type: WorkType<Message>,
//     handler: Arc<Handler>,
//     handler_id: HandlerId,
// }

/// The work to perform
pub struct Work<Message> {
    work_type: WorkType<Message>,
    handler: Arc<dyn IoHandler<Message>>,
    handler_id: HandlerId,
}

impl <Message> Work<Message> {
    pub fn new(work_type: WorkType<Message>, handler: Arc<dyn IoHandler<Message>>, handler_id: HandlerId) -> Self{
        Work {
            work_type,
            handler,
            handler_id
        }
    }
}

pub struct Wait {
    ready: Condvar,
    mutex: Mutex<()>
}

impl Wait {
    pub fn new() -> Self {
        Wait {
            ready: Condvar::new(),
            mutex: Mutex::new(()),
        }
    }
}

pub struct Worker {
    name: String,
    /// The wait signal when there is work to do
    wait: Arc<Wait>,
    /// The internal thread handler
    thread: Option<JoinHandle<()>>,
    /// Whether the worker has stopped working
    stopped: Arc<AtomicBool>,
}

impl Worker {
    pub fn new<Message: Send + Sync + 'static>(
        name: &str,
        stealer: crossbeam_deque::Stealer<Work<Message>>,
        wait: Arc<Wait>,
    ) -> Self {
        let w_name = format!("Worker-{}", name);
        let stopped = Arc::new(AtomicBool::new(false));
        let mut worker = Worker {
            name: w_name.clone(),
            wait: wait.clone(),
            thread: None,
            stopped: stopped.clone(),
        };

        worker.thread = Some(std::thread::Builder::new()
            .name(w_name)
            .stack_size(STACK_SIZE)
            .spawn(move || {
                while !stopped.load(Ordering::SeqCst) {
                    {
                        let mut l = wait.mutex.lock().unwrap();
                        wait.ready.wait_timeout(l, Duration::new(10, 0));
                    }

                    match stealer.steal() {
                        Steal::Empty => break,
                        Steal::Success(work) => {
                            Self::do_work(work);
                        },
                        Steal::Retry => {},
                    }
                }
            }).expect("Error creating worker thread"));

        worker
    }

    fn do_work<Message: Send + Sync + 'static>(work: Work<Message>) {
        match work.work_type {
            WorkType::Message(_) => {
                println!("handling work");
            }
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        log::debug!("terminating worker {}", self.name);
        self.stopped.store(true, Ordering::SeqCst);
        self.wait.mutex.lock().unwrap();
        self.wait.ready.notify_all();
        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
        log::info!("worker {} terminated", self.name);
    }
}

#[cfg(test)]
mod tests {
    use crate::worker::{Wait, Work};
    use crossbeam_deque;

    // #[test]
    // fn worker_works() {
    //     let wait = Wait::new();
    //     let w = crossbeam_deque::Worker::new_fifo();
    //     let stealer = w.stealer();
    // }
}
