use vm::{Bytes, Error, Exec, Ext, GasLeft};

mod gas;
mod memory;
mod stack;

struct CodeReader {
    /// The code to be executed
    code: Bytes,
    /// The position of where the code is
    position: usize
}
pub struct Interpreter {

}

impl Exec for Interpreter {
    fn exec(self: Box<Self>, ext: &mut dyn Ext) -> Result<GasLeft, Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn debug_works() {
        let a = 1;
        let b = 2;
    }
}