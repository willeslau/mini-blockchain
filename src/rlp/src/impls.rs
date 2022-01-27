use core::mem;
use crate::{Error, RLPStream, Rlp};
use crate::traits::{Encodable, Decodable};

pub fn decode_usize(bytes: &[u8]) -> Result<usize, Error> {
    match bytes.len() {
        l if l <= mem::size_of::<usize>() => {
            if bytes[0] == 0 {
                return Err(Error::RlpInvalidIndirection);
            }
            let mut res = 0usize;
            for (i, byte) in bytes.iter().enumerate().take(l) {
                let shift = (l - 1 - i) * 8;
                res += (*byte as usize) << shift;
            }
            Ok(res)
        }
        _ => Err(Error::RlpIsTooBig),
    }
}

impl Encodable for &str {
    fn encode(&self, stream: &mut RLPStream) {
        stream.write_iter(self.bytes())
    }
}

impl Encodable for Vec<u8> {
    fn encode(&self, stream: &mut RLPStream) {
        stream.write_iter(self.iter().cloned())
    }
}

macro_rules! impl_encodable_for_u {
	($name: ident) => {
		impl Encodable for $name {
			fn encode(&self, s: &mut RLPStream) {
				let leading_empty_bytes = self.leading_zeros() as usize / 8;
				let buffer = self.to_be_bytes();
                s.write_iter(buffer[leading_empty_bytes..].iter().cloned());
			}
		}
	};
}

macro_rules! impl_decodable_for_u {
	($name: ident) => {
		impl Decodable for $name {
			fn decode(rlp: &Rlp) -> Result<Self, Error> {
				rlp.decoder().decode_value(|bytes| match bytes.len() {
					0 | 1 => u8::decode(rlp).map(|v| v as $name),
					l if l <= mem::size_of::<$name>() => {
						if bytes[0] == 0 {
							return Err(Error::RlpInvalidIndirection);
						}
						let mut res = 0 as $name;
						for (i, byte) in bytes.iter().enumerate().take(l) {
							let shift = (l - 1 - i) * 8;
							res += (*byte as $name) << shift;
						}
						Ok(res)
					}
					_ => Err(Error::RlpIsTooBig),
				})
			}
		}
	};
}

impl Decodable for u8 {
    fn decode(rlp: &Rlp) -> Result<Self, Error> {
        rlp.decoder().decode_value(|bytes| match bytes.len() {
            1 if bytes[0] != 0 => Ok(bytes[0]),
            0 => Ok(0),
            1 => Err(Error::RlpInvalidIndirection),
            _ => Err(Error::RlpIsTooBig),
        })
    }
}

impl_encodable_for_u!(u64);
impl_encodable_for_u!(u8);
impl_decodable_for_u!(u64);


#[cfg(test)]
mod tests {
    use crate::{Decodable, Rlp, RLPStream};

    #[test]
    fn random_works() {
        let mut r = RLPStream::new();
        r.append(&u64::MAX);
        // let g = [8];
        // g[0..].into()
        assert_eq!(r.out(), vec![136, 255, 255, 255, 255, 255, 255, 255, 255]);
    }

    #[test]
    fn xcodable_for_u8_works() {
		let mut r = RLPStream::new();
		r.append(&u8::MAX);
		let o = r.out();

        let r = Rlp::new(&o);
		let u = u8::decode(&r).unwrap();
        assert_eq!(u, u8::MAX);
    }

    #[test]
    fn xcodable_for_u64_works() {
		let mut r = RLPStream::new();
		r.append(&u64::MAX);
		let o = r.out();

        let r = Rlp::new(&o);
		let u = u64::decode(&r).unwrap();
        assert_eq!(u, u64::MAX);
    }
}