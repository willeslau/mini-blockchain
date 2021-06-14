use transaction::Executable;

pub enum CollatorEvent {
    ExecutableAdded,
    InValid
}

/// Collator event listener
pub trait CollatorEventListener: Clone {
    /// Event published, handles the event accordingly
    fn on_event(&self, event: &CollatorEvent);
}

pub trait Collator: Clone {
    type Executable: Executable;

    /// Add a new event listener
    fn add_listener(&mut self, listener: String);
    /// Add executable to be collated
    fn add_executable(&mut self, executable: Self::Executable) -> bool;
    /// Dump all the valid executables
    fn dump(&self) -> Vec<Self::Executable>;
    /// Clear the executables
    fn clear(&mut self);
    /// Get the size of stored executables
    fn size(&self) -> usize;
}
