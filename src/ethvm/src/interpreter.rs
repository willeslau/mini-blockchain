use log;
use crate::cost::CostType;
use crate::error::Error;
use crate::gas::{GasMeter, InstructionGasRequirement};
use crate::instructions::Instruction;
use crate::memory::Memory;
use crate::stack::{Stack, VecStack};
use crate::types::{ActionParams, ActionValue, Bytes, CallType, Exec, Ext, GasLeft, ParamsType};

use common::{Address, BigEndianHash, H256, keccak, U256};
use crate::cache::JumpCache;

type ProgramCounter = usize;

struct CodeReader {
    /// The code to be executed
    code: Bytes,
    /// The position of where the code is
    position: ProgramCounter,
}

impl CodeReader {
    fn len(&self) -> usize {
        self.code.len()
    }

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

enum StepResult<M: Memory> {
    Continue,
    Error(Error),
    Success,
    Returned { memory: M, offset: usize, length: usize },
    Reverted,
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
                StepResult::Returned { .. } => return Ok(GasLeft::Known(U256::zero())),
                _ => todo!("impl other patterns")
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

    fn step(&mut self, ext: &mut dyn Ext) -> Result<StepResult<M>, Error> {
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

        self.exec_instruction(&instruction, ext)
    }

    fn validate_gas(&self) -> Result<(), Error> {
        Ok(())
    }

    fn validate_instruction(&self, instruction: &Instruction) -> Result<(), Error> {
        Ok(())
    }

    fn exec_instruction(&mut self, instruction: &Instruction, ext: &mut dyn Ext) -> Result<StepResult<M>, Error> {
       match instruction {
            Instruction::PUSH1 |
            Instruction::PUSH2 => {
                let bytes = instruction
                    .data_bytes()
                    .expect("invalid push read bytes. qed");
                let word = self.reader.read_word(bytes);
                log::debug!("{:?}: {:?}", instruction, word);

                self.stack.push(word);
            }
            Instruction::MSTORE => {
                let offset = self.stack.pop();
                let value = self.stack.pop();
                log::debug!("{:?}: offset {:?}, value: {:?}", instruction, offset, value);
                self.memory.write(offset, value);
            }
            Instruction::CALLVALUE => {
                self.stack.push(self.params.value.value());
                log::debug!("{:?}: value: {:?}", instruction, self.params.value.value());
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
                let v = self.stack.peek(index_from_top).clone();
                log::debug!("{:?}: value: {:?}", instruction, v);

                self.stack.push(v);
            },
            Instruction::ISZERO => {
                let v = Self::bool_to_u256(self.stack.pop().is_zero());
                log::debug!("{:?}: is_zero: {:?}", instruction, v);
                self.stack.push(v);
            },
            Instruction::JUMPI => {
                let dest = self.stack.pop().as_usize();
                let cond = Self::u256_to_bool(self.stack.pop());
                log::debug!("{:?}: cond: {:?}, dest: {:?}", instruction, cond, dest);
                self.process_jump(cond, ProgramCounter::from(dest))?;
            },
            Instruction::JUMPDEST => {
                log::debug!("{:?}", instruction);
            },
            Instruction::POP => {
                self.stack.pop();
                log::debug!("{:?}", instruction);
            },
            Instruction::MLOAD => {
                let offset = self.stack.pop();
                let v = self.memory.read(offset);
                log::debug!("{:?}, offset: {:?}, v: {:?}", instruction, offset, v);
                self.stack.push(v);
            },
            Instruction::CODESIZE => {
                log::debug!("{:?}, codelen: {:?}", instruction, self.reader.len());
                self.stack.push(U256::from(self.reader.len()));
            },
           Instruction::SUB => {
               let a = self.stack.pop();
               let b = self.stack.pop();
               log::debug!("{:?}, a: {:?}, b: {:?}, output: {:?}", instruction, a, b, a.overflowing_sub(b));
               self.stack.push(a.overflowing_sub(b).0);
           },
           Instruction::CODECOPY => {
               let dest_offset = self.stack.pop();
               let offset = self.stack.pop().as_usize();
               let end = offset + self.stack.pop().as_usize();
               log::debug!(
                   "{:?}, dest_offset: {:?}, offset: {:?}, end: {:?}",
                   instruction, dest_offset, offset, end
               );
               println!("{:x?}", &self.reader.code[offset..end]);
               self.memory.write_slice(dest_offset, &self.reader.code[offset..end])
           },
           Instruction::SWAP1
           | Instruction::SWAP2
           | Instruction::SWAP3
           | Instruction::SWAP4
           | Instruction::SWAP5
           | Instruction::SWAP6
           | Instruction::SWAP7
           | Instruction::SWAP8
           | Instruction::SWAP9
           | Instruction::SWAP10
           | Instruction::SWAP11
           | Instruction::SWAP12
           | Instruction::SWAP13
           | Instruction::SWAP14
           | Instruction::SWAP15
           | Instruction::SWAP16 => {
               let position = instruction
                   .swap_position()
                   .expect("swap_position always return some for SWAP* instructions");
               log::debug!("{:?}, position: {:?}", instruction, position);
               self.stack.swap_with_top(position);
           },
           Instruction::ADD => {
               let a = self.stack.pop();
               let b = self.stack.pop();
               let v = a.overflowing_add(b).0;
               log::debug!("{:?}, a: {:?}, b: {:?}, v: {:?}", instruction, a, b, v);
               self.stack.push(v);
           },
           Instruction::SSTORE => {
               let key = H256::from_uint(&self.stack.pop());
               let val = self.stack.pop();

               let current_val = ext.storage_at(&key)?.into_uint();
               // Increase refund for clear
               if ext.schedule().eip1283 {
                   todo!("impl this");
               } else {
                   if !current_val.is_zero() && val.is_zero() {
                       let sstore_clears_schedule = ext.schedule().sstore_refund_gas;
                       // TODO: find out what this does
                       ext.add_sstore_refund(sstore_clears_schedule);
                   }
               }
               ext.set_storage(key, BigEndianHash::from_uint(&val))?;
               ext.al_insert_storage_key(self.params.address, key);
               log::debug!("{:?}", instruction);
           },
           Instruction::CALLER => {
               let a = Self::address_to_u256(&self.params.sender);
               log::debug!("{:?}, address as u256: {:?}", instruction, a);
               self.stack.push(a);
           },
           Instruction::SHA3 => {
               let offset = self.stack.pop();
               let size = self.stack.pop();
               let k = keccak(self.memory.read_slice(offset, size));
               log::debug!("{:?}, offset: {:?}, size: {:?}, hash: {:?}", instruction, offset, size, k);
               self.stack.push(k.into_uint());
           },
           Instruction::RETURN => {
               let offset = self.stack.pop();
               let length = self.stack.pop();
               log::debug!("{:?}, offset: {:?}, length: {:?}", instruction, offset, length);
               let mut mem = core::mem::replace(&mut self.memory, Memory::empty());
               mem.write_slice(U256::zero(), self.memory.read_slice(offset, length));
               return Ok(StepResult::Returned {memory: mem, offset: offset.as_usize(), length: length.as_usize() })
           },
            _ => {
                log::debug!("{:?}", instruction);
                return Ok(StepResult::Error(Error::InvalidCommand));
            }
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

    fn address_to_u256(address: &Address) -> U256 {
        U256::from(address.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use crate::types::{ActionParams, Exec, FakeExt};
    use rustc_hex::FromHex;
    use env_logger;
    use common::{Address, U256};
    use crate::stack::Stack;

    #[test]
    fn run_code_works() {
        env_logger::init();

        let mut ext = FakeExt::new();
        let code = "608060405234801561001057600080fd5b5060405161027238038061027283398181016040528101906100329190610082565b816000819055508060018190555050506100c2565b600080fd5b6000819050919050565b61005f8161004c565b811461006a57600080fd5b50565b60008151905061007c81610056565b92915050565b6000806040838503121561009957610098610047565b5b60006100a78582860161006d565b92505060206100b88582860161006d565b9150509250929050565b6101a1806100d16000396000f3fe608060405234801561001057600080fd5b50600436106100415760003560e01c80630dbe671f146100465780634df7e3d01461006457806357fc036314610082575b600080fd5b61004e61009e565b60405161005b91906100df565b60405180910390f35b61006c6100a4565b60405161007991906100df565b60405180910390f35b61009c6004803603810190610097919061012b565b6100aa565b005b60005481565b60015481565b8060026000848152602001908152602001600020819055505050565b6000819050919050565b6100d9816100c6565b82525050565b60006020820190506100f460008301846100d0565b92915050565b600080fd5b610108816100c6565b811461011357600080fd5b50565b600081359050610125816100ff565b92915050565b60008060408385031215610142576101416100fa565b5b600061015085828601610116565b925050602061016185828601610116565b915050925092905056fea26469706673582212203cf30509388126a38220d62e1dc55643f8b148cea6ac8c7b98ccdd8f0ce02cc364736f6c634300080c003300000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001".from_hex().unwrap();
        let mut action_param = ActionParams::default();
        action_param.gas = U256::from(100);
        let mut interpreter = Interpreter::<Vec<u8>, usize>::new(code, action_param);
        interpreter.exec(&mut ext);

        // while interpreter.stack.size() > 0 {
        //     println!("{:?}", interpreter.stack.pop());
        // }
    }

    #[test]
    fn run_code_work() {
        env_logger::init();

        let mut ext = FakeExt::new();
        let code = "608060405234801561001057600080fd5b5060405160208061021783398101604090815290516000818155338152600160205291909120556101d1806100466000396000f3006080604052600436106100565763ffffffff7c010000000000000000000000000000000000000000000000000000000060003504166318160ddd811461005b57806370a0823114610082578063a9059cbb146100b0575b600080fd5b34801561006757600080fd5b506100706100f5565b60408051918252519081900360200190f35b34801561008e57600080fd5b5061007073ffffffffffffffffffffffffffffffffffffffff600435166100fb565b3480156100bc57600080fd5b506100e173ffffffffffffffffffffffffffffffffffffffff60043516602435610123565b604080519115158252519081900360200190f35b60005490565b73ffffffffffffffffffffffffffffffffffffffff1660009081526001602052604090205490565b600073ffffffffffffffffffffffffffffffffffffffff8316151561014757600080fd5b3360009081526001602052604090205482111561016357600080fd5b503360009081526001602081905260408083208054859003905573ffffffffffffffffffffffffffffffffffffffff85168352909120805483019055929150505600a165627a7a723058209a94330e3566febab4e903a73cf5b2a7674eca91ee95a8fcba4744635ead6c1500290000000000000000000000000000000000000000000000000000000000002710".from_hex().unwrap();
        let mut action_param = ActionParams::default();
        action_param.gas = U256::from(100);
        action_param.sender = Address::random();
        let mut interpreter = Interpreter::<Vec<u8>, usize>::new(code, action_param);
        interpreter.exec(&mut ext).unwrap();

        // while interpreter.stack.size() > 0 {
        //     println!("{:?}", interpreter.stack.pop());
        // }
    }
}
