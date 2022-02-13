//! Environment information for transaction execution.

use common::keccak;
use common::{Address, H256, U256};
// use ethjson;
use std::{cmp, sync::Arc};

type BlockNumber = u64;

/// Simple vector of hashes, should be at most 256 items large, can be smaller if being used
/// for a block whose number is less than 257.
pub type LastHashes = Vec<H256>;

/// Information concerning the execution environment for a message-call/contract-creation.
#[derive(Debug, Clone)]
pub struct EnvInfo {
    /// The block number.
    pub number: BlockNumber,
    /// The block author.
    pub author: Address,
    /// The block timestamp.
    pub timestamp: u64,
    /// The block difficulty.
    pub difficulty: U256,
    /// The block gas limit.
    pub gas_limit: U256,
    /// The last 256 block hashes.
    pub last_hashes: Arc<LastHashes>,
    /// The gas used.
    pub gas_used: U256,
    /// Block base fee.
    pub base_fee: Option<U256>,
}

impl Default for EnvInfo {
    fn default() -> Self {
        EnvInfo {
            number: 0,
            author: Address::default(),
            timestamp: 0,
            difficulty: 0.into(),
            gas_limit: 0.into(),
            last_hashes: Arc::new(vec![]),
            gas_used: 0.into(),
            base_fee: None,
        }
    }
}

// impl From<ethjson::vm::Env> for EnvInfo {
//     fn from(e: ethjson::vm::Env) -> Self {
//         let number = e.number.into();
//         EnvInfo {
//             number,
//             author: e.author.into(),
//             difficulty: e.difficulty.into(),
//             gas_limit: e.gas_limit.into(),
//             timestamp: e.timestamp.into(),
//             last_hashes: Arc::new(
//                 (1..cmp::min(number + 1, 257))
//                     .map(|i| keccak(format!("{}", number - i).as_bytes()))
//                     .collect(),
//             ),
//             gas_used: U256::default(),
//             base_fee: e.base_fee.map(|i| i.into()),
//         }
//     }
// }
