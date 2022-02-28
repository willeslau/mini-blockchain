/// Definition of the cost schedule and other parameterizations for the EVM.
#[derive(Debug, Default)]
pub struct Schedule {
    /// If Some(x): let limit = GAS * (x - 1) / x; let CALL's gas = min(requested, limit). let CREATE's gas = limit.
    /// If None: let CALL's gas = (requested > GAS ? [OOG] : GAS). let CREATE's gas = GAS
    pub sub_gas_cap_divisor: Option<usize>,
    /// Gas for used memory
    pub memory_gas: usize,
    /// Coefficient used to convert memory size to gas price for memory
    pub quad_coeff_div: usize,
    /// Gas prices for instructions in all tiers
    pub tier_step_gas: [usize; 8],
    /// TODO: read up on https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1283.md
    pub eip1283: bool,
    /// Gas refund for `SSTORE` clearing (when `storage!=0`, `new==0`)
    pub sstore_refund_gas: usize,
}

impl Schedule {
    fn new() -> Schedule {
        Schedule {
            tier_step_gas: [0, 2, 3, 5, 8, 10, 20, 0],
            memory_gas: 3,
            quad_coeff_div: 512,
            sub_gas_cap_divisor: None,
            eip1283: false,
            sstore_refund_gas: 15000
        }
    }
}
