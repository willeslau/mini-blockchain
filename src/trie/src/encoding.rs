pub(crate) const TERMINAL: u8 = 16;
const BITS_PER_NIBBLE: u8 = 4;

pub(crate) fn prefix_len(a: &[u8], b: &[u8]) -> usize {
    let mut i = 0;
    while i < a.len().min(b.len()) {
        if a[i] != b[i] {
            break;
        }
        i += 1;
    }
    i
}

pub(crate) fn key_bytes_to_hex(key: &[u8]) -> Vec<u8> {
    let l = key.len() * 2 + 1;
    let mut nibbles = vec![0; l];
    for (i, b) in key.iter().enumerate() {
        nibbles[i * 2] = *b / 16;
        nibbles[i * 2 + 1] = *b % 16;
    }
    nibbles[l - 1] = TERMINAL;
    nibbles
}

/*
func hexToCompact(hex []byte) []byte {
    terminator := byte(0)
    if hasTerm(hex) {
        terminator = 1
        hex = hex[:len(hex)-1]
    }
    buf := make([]byte, len(hex)/2+1)
    buf[0] = terminator << 5 // the flag byte
    if len(hex)&1 == 1 {
        buf[0] |= 1 << 4 // odd flag
        buf[0] |= hex[0] // first nibble is contained in the first byte
        hex = hex[1:]
    }
    decodeNibbles(hex, buf[1:])
    return buf
}
 */
pub fn hex_to_compact(hex: &[u8]) -> Vec<u8> {
    let mut terminator = 0u8;
    let (mut lo, mut hi) = (0, hex.len());
    if has_term(hex) {
        terminator = 1;
        hi -= 1;
    }
    let mut buf = Vec::with_capacity(hi / 2 + 1);
    // why push terminator << 5?
    buf.push(terminator << 5);
    if hi & 1 == 1 {
        buf[0] |= 1 << 4; // odd flag
        buf[0] |= hex[0]; // first nibble is contained in the first byte
        lo += 1;
    }
    decode_nibbles(hex, lo, hi, &mut buf);
    buf
}

/*
func decodeNibbles(nibbles []byte, bytes []byte) {
    for bi, ni := 0, 0; ni < len(nibbles); bi, ni = bi+1, ni+2 {
        bytes[bi] = nibbles[ni]<<4 | nibbles[ni+1]
    }
}
 */
pub fn decode_nibbles(nibbles: &[u8], lo: usize, hi: usize, bytes: &mut Vec<u8>) {
    let mut ni = lo;
    while ni < hi {
        bytes.push(nibbles[ni] << BITS_PER_NIBBLE | nibbles[ni + 1]);
        ni += 2;
    }
}

fn has_term(hex: &[u8]) -> bool {
    !hex.is_empty() && hex[hex.len() - 1] == TERMINAL
}

#[cfg(test)]
mod tests {
    use crate::encoding::{hex_to_compact, key_bytes_to_hex};

    #[test]
    fn key_bytes_to_hex_works() {
        let key = [0x12 as u8, 0x34, 0x56, 0x7];
        let r = key_bytes_to_hex(&key);
        assert_eq!(r, vec![1, 2, 3, 4, 5, 6, 0, 7, 16]);

        println!("{:?}", key_bytes_to_hex(b"foo"))
    }

    #[test]
    fn test_hex_to_compact() {
        /*
        {hex: []byte{}, compact: []byte{0x00}},
        {hex: []byte{16}, compact: []byte{0x20}},
        // odd length, no terminator
        {hex: []byte{1, 2, 3, 4, 5}, compact: []byte{0x11, 0x23, 0x45}},
        // even length, no terminator
        {hex: []byte{0, 1, 2, 3, 4, 5}, compact: []byte{0x00, 0x01, 0x23, 0x45}},
        // odd length, terminator
        {hex: []byte{15, 1, 12, 11, 8, 16 /*term*/}, compact: []byte{0x3f, 0x1c, 0xb8}},
        // even length, terminator
        {hex: []byte{0, 15, 1, 12, 11, 8, 16 /*term*/}, compact: []byte{0x20, 0x0f, 0x1c, 0xb8}},
         */
        assert_eq!(hex_to_compact(&[]), vec![0x00]);
        assert_eq!(hex_to_compact(&[16]), vec![0x20]);
        assert_eq!(hex_to_compact(&[1, 2, 3, 4, 5]), vec![0x11, 0x23, 0x45]);
    }
}
