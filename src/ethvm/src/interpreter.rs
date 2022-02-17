use crate::cost::CostType;
use crate::error::Error;
use crate::gas::{GasMeter, InstructionGasRequirement};
use crate::instructions::Instruction;
use crate::memory::Memory;
use crate::stack::{Stack, VecStack};
use crate::types::{Bytes, Exec, Ext, GasLeft};
use common::U256;

type ProgramCounter = usize;
const WORD_BYTES_SIZE: usize = 32;

// #[macro_export]
macro_rules! not_overflow {
    ($tuple: expr) => {
        if $tuple.1 {
            panic!("overflow");
        } else {
            $tuple.0
        }
    };
}

struct CodeReader {
    /// The code to be executed
    code: Bytes,
    /// The position of where the code is
    position: ProgramCounter,
}

impl CodeReader {
    fn instruction(&mut self) -> Instruction {
        let pos = self.position;
        self.position += 1;
        Instruction::from_u8(self.code[pos]).expect("invalid instruction code.qed")
    }

    fn done(&self) -> bool {
        self.position >= self.code.len()
    }

    fn read_word(&mut self, bytes: usize) -> U256 {
        let pos = self.position;
        self.position += bytes;
        let end = self.position.min(self.code.len());
        U256::from(&self.code[pos..end])
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

impl<M: Memory, G: CostType> Exec for Interpreter<M, G> {
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
            gas_meter: GasMeter::new(gas_limit),
        }
    }

    fn step(&mut self, ext: &mut dyn Ext) -> Result<StepResult, Error> {
        let instruction = self.reader.instruction();

        self.validate_instruction(&instruction)?;

        // NOTE: I think here is where Rust can handle relatively easier compared
        // NOTE: to other language. When handling some function that might involve
        // NOTE: multiple functions but also contain similar steps, i.e. in gas
        // NOTE: calculation, we might need to check the memory stack then expand
        // NOTE: the memory, it involves similar step to parse the instruction.
        // NOTE: In this case, we can use enum to handle and return all the
        // NOTE: parameters to avoid duplicated calculations.
        let requirement = self.gas_meter.instruction_requirement(&instruction, ext);
        self.validate_gas(requirement.gas())?;

        // expand memory to the required size
        if let InstructionGasRequirement::Mem {
            gas: default,
            mem_gas,
            mem_size,
        } = requirement
        {
            self.memory.expand(mem_size);
        }

        self.exec_instruction(&instruction)
    }

    fn validate_gas(&self, gas: &G) -> Result<(), Error> {
        Ok(())
    }

    fn validate_instruction(&self, instruction: &Instruction) -> Result<(), Error> {
        Ok(())
    }

    fn exec_instruction(&mut self, instruction: &Instruction) -> Result<StepResult, Error> {
        let mut r = match instruction {
            Instruction::PUSH1 => {
                let bytes = instruction
                    .data_bytes()
                    .expect("invalid push read bytes. qed");
                let word = self.reader.read_word(bytes);
                self.stack.push(word);
                StepResult::Continue
            }
            _ => StepResult::Success,
        };

        if self.reader.done() {
            r = StepResult::Success;
        }

        Ok(r)
    }
}

fn mem_add_size(current: usize, to_add: usize) -> usize {
    current.checked_add(to_add).expect("oom")
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use crate::types::{Exec, FakeExt};
    use rustc_hex::FromHex;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn push_works() {
        let mut ext = FakeExt::new();
        let code = "6080604052".from_hex().unwrap();
        let mut interpreter = Interpreter::<Vec<u8>, usize>::new(code, 100000);
        interpreter.exec(&mut ext);
    }
}
