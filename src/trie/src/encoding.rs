/*
// NOTE: This function just converts the bytes into their corresponding nibbles.
// NOTE: At the same time, appending the terminal symbol to the back.
func keybytesToHex(str []byte) []byte {
    l := len(str)*2 + 1
    var nibbles = make([]byte, l)
    for i, b := range str {
        nibbles[i*2] = b / 16
        nibbles[i*2+1] = b % 16
    }
    nibbles[l-1] = 16
    return nibbles
}
 */

const TERMINAL: u8 = 16;

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

#[cfg(test)]
mod tests {
    use crate::encoding::key_bytes_to_hex;

    #[test]
    fn key_bytes_to_hex_works() {
        let key = [0x12 as u8, 0x34, 0x56, 0x7];
        let r = key_bytes_to_hex(&key);
        assert_eq!(r, vec![1, 2, 3, 4, 5, 6, 0, 7, 16]);
    }
}
