use crate::cost::CostType;
use crate::error::Error;
use crate::gas::{GasMeter, InstructionGasRequirement};
use crate::instructions::Instruction;
use crate::memory::Memory;
use crate::stack::{Stack, VecStack};
use crate::types::{ActionParams, ActionValue, Bytes, CallType, Exec, Ext, GasLeft, ParamsType};

use common::{Address, H256, U256};
use crate::cache::JumpCache;

type ProgramCounter = usize;

struct CodeReader {
    /// The code to be executed
    code: Bytes,
    /// The position of where the code is
    position: ProgramCounter,
}

impl CodeReader {
    fn set_pc(&mut self, pc: ProgramCounter) {
        if pc.as_usize() >= self.code.len() { panic!("invalid program counter"); }
        self.position = pc;
    }

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

/// ActionParams without code, so that it can be feed into CodeReader.
#[derive(Debug)]
struct InterpreterParams {
    /// Address of currently executed code.
    pub code_address: Address,
    /// Hash of currently executed code.
    pub code_hash: Option<H256>,
    /// Receive address. Usually equal to code_address,
    /// except when called using CALLCODE.
    pub address: Address,
    /// Sender of current part of the transaction.
    pub sender: Address,
    /// Transaction initiator.
    pub origin: Address,
    /// Gas paid up front for transaction execution
    pub gas: U256,
    /// Gas price.
    pub gas_price: U256,
    /// Transaction value.
    pub value: ActionValue,
    /// Input data.
    pub data: Option<Bytes>,
    /// Type of call
    pub call_type: CallType,
    /// Param types encoding
    pub params_type: ParamsType,
}

impl From<ActionParams> for InterpreterParams {
    fn from(params: ActionParams) -> Self {
        InterpreterParams {
            code_address: params.code_address,
            code_hash: params.code_hash,
            address: params.address,
            sender: params.sender,
            origin: params.origin,
            gas: params.gas,
            gas_price: params.gas_price,
            value: params.value,
            data: params.data,
            call_type: params.call_type,
            params_type: params.params_type,
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
    params: InterpreterParams,
    jump_cache: Option<JumpCache>,
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
    pub fn new(code: Vec<u8>, action_param: ActionParams) -> Self {
        let reader = CodeReader { code, position: 0 };
        let gas = G::from_u256(action_param.gas).expect("cannot parse gas");
        Self {
            reader,
            stack: VecStack::with_capacity(1024, U256::zero()),
            memory: M::empty(),
            gas_meter: GasMeter::new(gas),
            params: InterpreterParams::from(action_param),
            jump_cache: None
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
        let requirement = self.gas_meter.instruction_requirement(&instruction, ext, &self.stack);
        self.gas_meter.update(&requirement)?;
        self.validate_gas()?;

        // expand memory to the required size
        if let InstructionGasRequirement::Mem {
            mem_size,
            ..
        } = requirement
        {
            self.memory.resize(mem_size + self.memory.size());
        }

        self.exec_instruction(&instruction)
    }

    fn validate_gas(&self) -> Result<(), Error> {
        Ok(())
    }

    fn validate_instruction(&self, instruction: &Instruction) -> Result<(), Error> {
        Ok(())
    }

    fn exec_instruction(&mut self, instruction: &Instruction) -> Result<StepResult, Error> {
        let mut r = match instruction {
            Instruction::PUSH1 |
            Instruction::PUSH2 => {
                let bytes = instruction
                    .data_bytes()
                    .expect("invalid push read bytes. qed");
                let word = self.reader.read_word(bytes);
                self.stack.push(word);
            }
            Instruction::MSTORE => {
                let offset = self.stack.pop();
                let value = self.stack.pop();
                self.memory.write(offset, value);
            }
            Instruction::CALLVALUE => {
                self.stack.push(self.params.value.value());
            },
            Instruction::DUP1
            | Instruction::DUP2
            | Instruction::DUP3
            | Instruction::DUP4
            | Instruction::DUP5
            | Instruction::DUP6
            | Instruction::DUP7
            | Instruction::DUP8
            | Instruction::DUP9
            | Instruction::DUP10
            | Instruction::DUP11
            | Instruction::DUP12
            | Instruction::DUP13
            | Instruction::DUP14
            | Instruction::DUP15
            | Instruction::DUP16 => {
                let index_from_top = instruction.dup_position().expect("invalid operation.qed");
                self.stack.push(self.stack.peek(index_from_top).clone());
            },
            Instruction::ISZERO => {
                let v = Self::bool_to_u256(self.stack.pop().is_zero());
                self.stack.push(v);
            },
            Instruction::JUMPI => {
                let dest = self.stack.pop().as_usize();
                let cond = Self::u256_to_bool(self.stack.pop());
                self.process_jump(cond, ProgramCounter::from(dest))?;
            },
            Instruction::JUMPDEST => {},
            Instruction::POP => {
                self.stack.pop();
            },
            Instruction::MLOAD => {
                let offset = self.stack.pop();
                let v = self.memory.read(offset);
                self.stack.push(v);
            },
            Instruction::CODESIZE => {
                self.stack.push(U256::from(self.reader.len()));
            },
            _ => return Ok(StepResult::Error(Error::InvalidCommand))
        };

        if self.reader.done() { return Ok(StepResult::Success); }

        Ok(StepResult::Continue)
    }

    fn process_jump(&mut self, cond: bool, dest: ProgramCounter) -> Result<(), Error> {
        if self.jump_cache.is_none() {
            self.jump_cache = Some(JumpCache::new(&self.reader.code));
        }

        if !cond {
            self.reader.set_pc(ProgramCounter::from(self.reader.position + 1));
            return Ok(());
        }

        match &self.jump_cache {
            Some(cache) => {
                cache.valid_jump_dest(dest.as_usize())?;
                self.reader.set_pc(dest);
            },
            None => panic!("should not happen"),
        }
        Ok(())
    }

    fn bool_to_u256(val: bool) -> U256 {
        if val {
            U256::one()
        } else {
            U256::zero()
        }
    }

    fn u256_to_bool(val: U256) -> bool { !val.is_zero() }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use crate::types::{ActionParams, Exec, FakeExt};
    use rustc_hex::FromHex;
    use common::U256;
    use crate::stack::Stack;

    #[test]
    fn push_works() {
        let mut ext = FakeExt::new();
        let code = "608060405234801561001057600080fd5b50604051610272".from_hex().unwrap();
        let mut action_param = ActionParams::default();
        action_param.gas = U256::from(100);
        let mut interpreter = Interpreter::<Vec<u8>, usize>::new(code, action_param);
        interpreter.exec(&mut ext);

        while interpreter.stack.size() > 0 {
            println!("{:?}", interpreter.stack.pop());
        }
    }
}
