use crate::error::Error;

pub const U64_LEN: usize = 8;

/// Convert vec of u8 into a u64
pub fn vec_to_u64_le(nums: Vec<u8>) -> Result<u64, Error> {
    if nums.len() != U64_LEN {
        Err(Error::InvalidLength)
    } else {
        let mut v = [0u8; U64_LEN];
        v.copy_from_slice(&nums[..]);
        Ok(u64::from_le_bytes(v))
    }
}

#[cfg(test)]
mod tests {
    use crate::vec_to_u64_le;

    #[test]
    fn vec_to_u64_le_works() {
        let n = 1000u64;
        let v = n.to_le_bytes().to_vec();
        assert_eq!(n, vec_to_u64_le(v).unwrap())
    }
}