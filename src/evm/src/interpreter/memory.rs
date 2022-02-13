use common::U256;
use vm::ReturnData;

const MAX_RETURN_WASTE_BYTES: usize = 16384;

pub trait Memory {
    /// Retrieve current size of the memory
    fn size(&self) -> usize;
    /// Resize (shrink or expand) the memory to specified size (fills 0)
    fn resize(&mut self, new_size: usize);
    /// Resize the memory only if its smaller
    fn expand(&mut self, new_size: usize);
    /// Write single byte to memory
    fn write_byte(&mut self, offset: U256, value: U256);
    /// Write a word to memory. Does not resize memory!
    fn write(&mut self, offset: U256, value: U256);
    /// Read a word from memory
    fn read(&self, offset: U256) -> U256;
    /// Write slice of bytes to memory. Does not resize memory!
    fn write_slice(&mut self, offset: U256, bytes: &[u8]);
    /// Retrieve part of the memory between offset and offset + size
    fn read_slice(&self, offset: U256, size: U256) -> &[u8];
    /// Retrieve writeable part of memory
    fn writeable_slice(&mut self, offset: U256, size: U256) -> &mut [u8];
    /// Convert memory into return data.
    fn into_return_data(self, offset: U256, size: U256) -> ReturnData;
}

fn is_valid_range(offset: usize, size: usize) -> bool {
    match offset.checked_add(size) {
        None => false,
        Some(_) => size > 0,
    }
}

impl Memory for Vec<u8> {
    fn size(&self) -> usize {
        self.len()
    }

    fn resize(&mut self, new_size: usize) {
        self.resize(new_size, 0u8)
    }

    fn expand(&mut self, new_size: usize) {
        if new_size > self.len() {
            Memory::resize(self, new_size);
        }
    }

    fn write_byte(&mut self, offset: U256, value: U256) {
        let offset = offset.low_u64() as usize;
        self[offset] = value.low_u64() as u8;
    }

    fn write(&mut self, offset: U256, value: U256) {
        let offset = offset.low_u64() as usize;
        value.to_big_endian(&mut self[offset..offset + 32])
    }

    fn read(&self, offset: U256) -> U256 {
        let offset = offset.low_u64() as usize;
        U256::from(&self[offset..offset + 32])
    }

    fn write_slice(&mut self, offset: U256, slice: &[u8]) {
        if !slice.is_empty() {
            let offset = offset.low_u64() as usize;
            self[offset..offset + slice.len()].copy_from_slice(slice);
        }
    }

    fn read_slice(&self, offset: U256, size: U256) -> &[u8] {
        let off = offset.low_u64() as usize;
        let size = size.low_u64() as usize;
        if !is_valid_range(off, size) {
            &self[0..0]
        } else {
            &self[off..off + size]
        }
    }

    fn writeable_slice(&mut self, offset: U256, size: U256) -> &mut [u8] {
        let off = offset.low_u64() as usize;
        let len = size.low_u64() as usize;
        if !is_valid_range(off, len) {
            &mut self[0..0]
        } else {
            &mut self[off..off + len]
        }
    }

    fn into_return_data(mut self, offset: U256, size: U256) -> ReturnData {
        let mut off = offset.low_u64() as usize;
        let len = size.low_u64() as usize;

        if !is_valid_range(off, len) {
            return ReturnData::empty();
        }

        if self.len() - len > MAX_RETURN_WASTE_BYTES {
            if off == 0 {
                self.truncate(len);
                self.shrink_to_fit();
            } else {
                self = self[off..off + len].to_vec();
                off = 0;
            }
        }

        ReturnData::new(self, off, len)
    }
}

#[cfg(test)]
mod tests {
    use super::Memory;
    use common::U256;

    #[test]
    fn test_memory_read_and_write() {
        // given
        let mem: &mut dyn Memory = &mut vec![];
        mem.resize(0x80 + 32);

        // when
        mem.write(U256::from(0x80), U256::from(0xabcdef));

        // then
        assert_eq!(mem.read(U256::from(0x80)), U256::from(0xabcdef));
    }

    #[test]
    fn test_memory_read_and_write_byte() {
        // given
        let mem: &mut dyn Memory = &mut vec![];
        mem.resize(32);

        // when
        mem.write_byte(U256::from(0x1d), U256::from(0xab));
        mem.write_byte(U256::from(0x1e), U256::from(0xcd));
        mem.write_byte(U256::from(0x1f), U256::from(0xef));

        // then
        assert_eq!(mem.read(U256::from(0x00)), U256::from(0xabcdef));
    }

    #[test]
    fn test_memory_read_slice_and_write_slice() {
        let mem: &mut dyn Memory = &mut vec![];
        mem.resize(32);

        {
            let slice = "abcdefghijklmnopqrstuvwxyz012345".as_bytes();
            mem.write_slice(U256::from(0), slice);

            assert_eq!(mem.read_slice(U256::from(0), U256::from(32)), slice);
        }

        // write again
        {
            let slice = "67890".as_bytes();
            mem.write_slice(U256::from(0x1), slice);

            assert_eq!(
                mem.read_slice(U256::from(0), U256::from(7)),
                "a67890g".as_bytes()
            );
        }

        // write empty slice out of bounds
        {
            let slice = [];
            mem.write_slice(U256::from(0x1000), &slice);
            assert_eq!(mem.size(), 32);
        }
    }
}
