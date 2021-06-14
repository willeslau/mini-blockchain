use crate::traits::{Collator, CollatorEvent};

#[derive(Clone)]
pub struct DefaultCollator<Executable: transaction::Executable> {
    listeners: Vec<String>,
    executables: Vec<Executable>,
}

impl <Executable: transaction::Executable> DefaultCollator<Executable> {
    pub fn new() -> Self {
        DefaultCollator{
            listeners: vec![],
            executables: vec![]
        }
    }

    /// Publish event to the listeners
    /// TODO: add implementation, refer to:
    ///     https://www.alexgarrett.tech/blog/article/observer-pattern-in-rust/
    ///     https://doc.rust-lang.org/book/ch17-02-trait-objects.html
    fn publish_event(&self, _event: &CollatorEvent) {
    }
}

impl <Executable: transaction::Executable> Collator for DefaultCollator<Executable> {
    type Executable = Executable;

    fn add_listener(&mut self, listener: String) {
        self.listeners.push(listener);
    }

    fn add_executable(&mut self, executable: Self::Executable) -> bool {
        if !executable.is_valid() {
            self.publish_event(&CollatorEvent::InValid);
            return false;
        }
        self.executables.push(executable);
        self.publish_event(&CollatorEvent::ExecutableAdded);
        true
    }

    fn dump(&self) -> Vec<Self::Executable> {
        self.executables.clone()
    }

    fn clear(&mut self) {
        self.executables = vec![];
    }

    fn size(&self) -> usize {
        self.executables.len()
    }
}