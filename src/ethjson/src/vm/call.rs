//! Vm call deserialization.

use crate::{bytes::Bytes, hash::Address, maybe::MaybeEmpty, uint::Uint};

/// Vm call deserialization.
#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Call {
    /// Call data.
    pub data: Bytes,
    /// Call destination.
    pub destination: MaybeEmpty<Address>,
    /// Gas limit.
    pub gas_limit: Uint,
    /// Call value.
    pub value: Uint,
}

#[cfg(test)]
mod tests {
    use crate::{hash::Address, maybe::MaybeEmpty, uint::Uint, vm::Call};
    use common::{H160 as Hash160, U256};

    use serde_json;
    use std::str::FromStr;

    #[test]
    fn call_deserialization_empty_dest() {
        let s = r#"{
			"data" : "0x1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff",
			"destination" : "",
			"gasLimit" : "0x1748766aa5",
			"value" : "0x00"
		}"#;
        let call: Call = serde_json::from_str(s).unwrap();

        assert_eq!(
            &call.data[..],
            &[
                0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0x44, 0x44, 0x55, 0x55, 0x66, 0x66, 0x77, 0x77,
                0x88, 0x88, 0x99, 0x99, 0x00, 0x00, 0xaa, 0xaa, 0xbb, 0xbb, 0xcc, 0xcc, 0xdd, 0xdd,
                0xee, 0xee, 0xff, 0xff
            ]
        );

        assert_eq!(call.destination, MaybeEmpty::None);
        assert_eq!(call.gas_limit, Uint(U256::from(0x1748766aa5u64)));
        assert_eq!(call.value, Uint(U256::from(0)));
    }

    #[test]
    fn call_deserialization_full_dest() {
        let s = r#"{
			"data" : "0x1234",
			"destination" : "5a39ed1020c04d4d84539975b893a4e7c53eab6c",
			"gasLimit" : "0x1748766aa5",
			"value" : "0x00"
		}"#;

        let call: Call = serde_json::from_str(s).unwrap();

        assert_eq!(&call.data[..], &[0x12, 0x34]);
        assert_eq!(
            call.destination,
            MaybeEmpty::Some(Address(
                Hash160::from_str("5a39ed1020c04d4d84539975b893a4e7c53eab6c").unwrap()
            ))
        );
        assert_eq!(call.gas_limit, Uint(U256::from(0x1748766aa5u64)));
        assert_eq!(call.value, Uint(U256::from(0)));
    }
}
