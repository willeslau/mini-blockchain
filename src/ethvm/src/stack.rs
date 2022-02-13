use crate::instructions;

/// Stack trait with VM-friendly API
pub trait Stack<T> {
    /// Returns `Stack[len(Stack) - no_from_top]`
    fn peek(&self, no_from_top: usize) -> &T;
    /// Swaps Stack[len(Stack)] and Stack[len(Stack) - no_from_top]
    fn swap_with_top(&mut self, no_from_top: usize);
    /// Returns true if Stack has at least `no_of_elems` elements
    fn has(&self, no_of_elems: usize) -> bool;
    /// Get element from top and remove it from Stack. Panics if stack is empty.
    fn pop(&mut self) -> T;
    /// Get (up to `instructions::MAX_NO_OF_TOPICS`) elements from top and remove them from Stack. Panics if stack is empty.
    fn pop_n(&mut self, no_of_elems: usize) -> &[T];
    /// Add element on top of the Stack
    fn push(&mut self, elem: T);
    /// Get number of elements on Stack
    fn size(&self) -> usize;
    /// Returns all data on stack.
    fn peek_all(&self, no_from_top: usize) -> &[T];
}

pub struct VecStack<S> {
    stack: Vec<S>,
    logs: [S; instructions::MAX_NO_OF_TOPICS],
}

impl<S: Copy> VecStack<S> {
    pub fn with_capacity(capacity: usize, zero: S) -> Self {
        Self {
            stack: Vec::with_capacity(capacity),
            logs: [zero; instructions::MAX_NO_OF_TOPICS],
        }
    }
}

impl<S> Stack<S> for VecStack<S> {
    fn peek(&self, no_from_top: usize) -> &S {
        &self.stack[self.stack.len() - no_from_top - 1]
    }

    fn swap_with_top(&mut self, no_from_top: usize) {
        let l1 = self.stack.len() - 1;
        let l2 = self.stack.len() - no_from_top - 1;
        self.stack.swap(l1, l2);
    }

    fn has(&self, no_of_elems: usize) -> bool {
        self.stack.len() >= no_of_elems
    }

    fn pop(&mut self) -> S {
        self.stack.pop().expect("stack empty")
    }

    fn pop_n(&mut self, no_of_elems: usize) -> &[S] {
        let n = no_of_elems.min(instructions::MAX_NO_OF_TOPICS);
        for i in 0..n {
            self.logs[i] = self.stack.pop().expect("stack empty");
        }
        &self.logs
    }

    fn push(&mut self, elem: S) {
        self.stack.push(elem);
    }

    fn size(&self) -> usize {
        self.stack.len()
    }

    fn peek_all(&self, no_from_top: usize) -> &[S] {
        assert!(
            self.stack.len() >= no_from_top,
            "peek_top asked for more items than exist. qed."
        );
        &self.stack[self.stack.len() - no_from_top..self.stack.len()]
    }
}
