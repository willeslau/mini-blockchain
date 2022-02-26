//! EVM call types.

use rlp::{Decodable, Error, Encodable, Rlp, RLPStream};

/// The type of the call-like instruction.
#[derive(Debug, PartialEq, Clone)]
pub enum CallType {
    /// Not a CALL.
    None,
    /// CALL.
    Call,
    /// CALLCODE.
    CallCode,
    /// DELEGATECALL.
    DelegateCall,
    /// STATICCALL
    StaticCall,
}

impl Encodable for CallType {
    fn encode(&self, s: &mut RLPStream) {
        let v = match *self {
            CallType::None => 0u32,
            CallType::Call => 1,
            CallType::CallCode => 2,
            CallType::DelegateCall => 3,
            CallType::StaticCall => 4,
        };
        Encodable::encode(&v, s);
    }
}

impl Decodable for CallType {
    fn decode(rlp: &Rlp) -> Result<Self, Error> {
        rlp.as_val().and_then(|v| {
            Ok(match v {
                0u32 => CallType::None,
                1 => CallType::Call,
                2 => CallType::CallCode,
                3 => CallType::DelegateCall,
                4 => CallType::StaticCall,
                _ => return Err(Error::Custom("Invalid value of CallType item")),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CallType;
    use rlp::*;

    #[test]
    fn encode_call_type() {
        let ct = CallType::Call;

        let mut s = RLPStream::new_list(2);
        s.append(&ct);
        s.append(&ct);
        s.out();
    }

    // #[test]
    // fn should_encode_and_decode_call_type() {
    //     let original = CallType::Call;
    //     let encoded = encode(&original);
    //     let decoded = decode(&encoded).expect("failure decoding CallType");
    //     assert_eq!(original, decoded);
    // }
}
