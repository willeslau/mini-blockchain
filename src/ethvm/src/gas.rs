use crate::cost::CostType;
use crate::error::Error;
use crate::instructions::{Instruction, InstructionInfo};
use crate::stack::VecStack;

use common::{Address, U256};
use std::cmp;
use crate::types::{Ext, Schedule};

macro_rules! overflowing {
    ($x: expr) => {{
        let (v, overflow) = $x;
        if overflow {
            return Err(Error::OutOfGas);
        }
        v
    }};
}

enum Request<Cost: CostType> {
    Gas(Cost),
    GasMem(Cost, Cost),
    GasMemProvide(Cost, Cost, Option<U256>),
    GasMemCopy(Cost, Cost, Cost),
}

pub struct InstructionRequirements<Cost> {
    pub gas_cost: Cost,
    pub provide_gas: Option<Cost>,
    pub memory_total_gas: Cost,
    pub memory_required_size: usize,
}

pub(crate) struct GasMeter<Gas: CostType> {
    current_gas: Gas,
    current_mem_gas: Gas,
}

impl<Gas: CostType> GasMeter<Gas> {
    pub fn new(current_gas: Gas) -> Self {
        GasMeter {
            current_gas,
            current_mem_gas: Gas::from(0),
        }
    }

    pub fn verify_gas(&self, gas_cost: &Gas) -> Result<(), Error> {
        match &self.current_gas < gas_cost {
            true => Err(Error::OutOfGas),
            false => Ok(()),
        }
    }

    /// How much gas is provided to a CALL/CREATE, given that we need to deduct `needed` for
    /// this operation and that we `requested` some.
    pub fn gas_call_or_create(
        &self,
        schedule: &Schedule,
        needed: Gas,
        requested: Option<U256>,
    ) -> Result<Gas, Error> {
        // Try converting requested gas to `Gas` (`U256/u64`)
        // but in EIP150 even if we request more we should never fail from OOG
        let requested = requested.map(Gas::from_u256);

        match schedule.sub_gas_cap_divisor {
            Some(cap_divisor) if self.current_gas >= needed => {
                let gas_remaining = self.current_gas - needed;
                // TODO: what does this do? Reserve some gas for?
                let max_gas_provided = match cap_divisor {
                    64 => gas_remaining - (gas_remaining >> 6),
                    cap_divisor => gas_remaining - gas_remaining / Gas::from(cap_divisor),
                };

                if let Some(Ok(r)) = requested {
                    Ok(cmp::min(r, max_gas_provided))
                } else {
                    Ok(max_gas_provided)
                }
            }
            _ => {
                if let Some(r) = requested {
                    r
                } else if self.current_gas >= needed {
                    Ok(self.current_gas - needed)
                } else {
                    Ok(0.into())
                }
            }
        }
    }

    fn mem_gas_cost(
        &self,
        schedule: &Schedule,
        current_mem_size: usize,
        mem_size: &Gas,
    ) -> Result<(Gas, Gas, usize), Error> {
        // This calculates the memory usage for gas.
        // According to the yellow paper, it is:
        //     G = Gmemory * a + a ^ 2 / 512
        // where a is the number of 256-bit words allocated, Gmemory = schedule.memory_gas
        // and Gmemory should be 3.
        let gas_for_mem = |mem_size: Gas| {
            // TODO: This is only 2^5 = 32, not 256. Unless, memory is
            // TODO: implemented as Vec<u8>, this would be 256 bits.
            // TODO: Confirm above?
            let s = mem_size >> 5;
            // s * memory_gas + s * s / quad_coeff_div
            let a = overflowing!(s.overflow_mul(Gas::from(schedule.memory_gas)));

            // Calculate s*s/quad_coeff_div
            assert_eq!(schedule.quad_coeff_div, 512);
            let b = overflowing!(s.overflow_mul_shr(s, 9));

            Ok(overflowing!(a.overflow_add(b)))
        };

        let current_mem_size = Gas::from(current_mem_size);
        let req_mem_size_rounded = overflowing!(to_word_size(*mem_size)) << 5;

        let (mem_gas_cost, new_mem_gas) = if req_mem_size_rounded > current_mem_size {
            let new_mem_gas = gas_for_mem(req_mem_size_rounded)?;
            (new_mem_gas - self.current_mem_gas, new_mem_gas)
        } else {
            (Gas::from(0), self.current_mem_gas)
        };

        Ok((mem_gas_cost, new_mem_gas, req_mem_size_rounded.as_usize()))
    }

    /// Determine how much gas is used by the given instruction, given the machine's state.
    ///
    /// We guarantee that the final element of the returned tuple (`provided`) will be `Some`
    /// iff the `instruction` is one of `CREATE`, or any of the `CALL` variants. In this case,
    /// it will be the amount of gas that the current context provides to the child context.
    pub fn requirements(
        &mut self,
        ext: &dyn Ext,
        instruction: Instruction,
        info: &InstructionInfo,
        stack: &VecStack<U256>,
        current_address: &Address,
        current_mem_size: usize,
    ) -> Result<InstructionRequirements<Gas>, Error> {
        let schedule = ext.schedule();
        let tier = info.tier.idx();
        let default_gas = Gas::from(schedule.tier_step_gas[tier]);

        let cost = match instruction {
            _ => Request::Gas(default_gas),
        };

        Ok(match cost {
            Request::Gas(gas) => InstructionRequirements {
                gas_cost: gas,
                provide_gas: None,
                memory_required_size: 0,
                memory_total_gas: self.current_mem_gas,
            },
            _ => todo!(),
        })
    }
}

#[inline]
fn add_gas_usize<Gas: CostType>(value: Gas, num: usize) -> (Gas, bool) {
    value.overflow_add(Gas::from(num))
}

#[inline]
fn to_word_size<Gas: CostType>(value: Gas) -> (Gas, bool) {
    let (gas, overflow) = add_gas_usize(value, 31);
    if overflow {
        return (gas, overflow);
    }
    (gas >> 5, false)
}

// #[cfg(test)]
// mod tests {
//     use crate::gas::GasMeter;
//     use crate::instructions::Instruction;
//     use crate::stack::VecStack;
//     use common::{Address, U256};
//
//     #[test]
//     fn requirements_works() {
//         let mut gas_meter = GasMeter::new(1000);
//         let instruction = Instruction::from_u8(1).unwrap();
//         let stack = VecStack::with_capacity(1024, U256::zero());
//         let address = Address::random();
//         let mem_size = 100;
//         gas_meter
//             .requirements(
//                 &ext,
//                 instruction,
//                 instruction.info(),
//                 &stack,
//                 &address,
//                 mem_size,
//             )
//             .unwrap();
//     }
// }
