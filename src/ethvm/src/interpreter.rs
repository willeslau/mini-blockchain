use crate::cost::CostType;
use crate::error::Error;
use crate::instructions::Instruction;
use crate::memory::Memory;
use crate::stack::VecStack;
use crate::types::{Exec, Ext, GasLeft};
use crate::gas::GasMeter;
use common::U256;

struct CodeReader {
    /// The code to be executed
    code: Vec<u8>,
    /// The position of where the code is
    position: usize,
}

impl CodeReader {
    fn instruction(&self) -> Instruction {
        Instruction::from_u8(self.code[self.position]).expect("invalid instruction code.qed")
    }

    fn bytes(&self, instruction: &Instruction) -> &[u8] {
        match instruction {
            _ => &self.code,
        }
    }

    fn read_words(&self, instruction: &Instruction) -> Vec<U256> {
        vec![U256::zero()]
    }

    fn advance(&mut self, bytes: usize) {
        self.position += bytes;
        if self.position > self.code.len() {
            panic!("invalid source code len. qed.")
        }
    }
}

enum StepResult {
    Continue,
    Error(Error),
    Success,
}

pub struct Interpreter<M: Memory, G: CostType> {
    reader: CodeReader,
    stack: VecStack<U256>,
    memory: M,
    gas_meter: GasMeter<G>,
}

impl <M: Memory, G: CostType> Exec for Interpreter<M, G> {
    fn exec(&mut self, ext: &mut dyn Ext) -> Result<GasLeft, Error> {
        loop {
            match self.step(ext)? {
                StepResult::Continue => {}
                StepResult::Error(e) => return Err(e),
                StepResult::Success => return Ok(GasLeft::Known(U256::zero())),
            };
        }
    }
}

impl<M: Memory, G: CostType> Interpreter<M, G> {
    pub fn new(code: Vec<u8>, gas_limit: G) -> Self {
        let reader = CodeReader { code, position: 0 };
        Self {
            reader,
            stack: VecStack::with_capacity(1024, U256::zero()),
            memory: M::empty(),
            gas_meter: GasMeter::new(gas_limit)
        }
    }

    fn step(&mut self, ext: &mut dyn Ext) -> Result<StepResult, Error> {
        let instruction = self.reader.instruction();

        self.validate_instruction(&instruction)?;

        let gas = self.derive_gas();
        sekf.validate_gas(&gas)?;

        // expand memory to the required size
        self.memory.expand(0);

        self.exec_instruction(&instruction)
    }

    fn validate_instruction(&self, instruction: &Instruction) -> Result<(), Error> {
        Ok(())
    }

    fn derive_gas(&self) -> G {
        G::from(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn debug_works() {
        let a = 1;
        let b = 2;
        let c = a + b;
        // let i = Interpreter::new();
        // thread::sleep(Duration::from_millis(1000));
    }
}
