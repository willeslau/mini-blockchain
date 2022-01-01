const STR_OFFSET: u8 = 0x80;
const LIST_OFFSET: u8 = 0xc0;
const LEN_CUTOFF: u8 = 55;

pub struct RPLStream {
    data: Vec<u8>,
    /// The index of the list currently being inserted
    appending_list: Vec<(usize, usize)>,
}

impl RPLStream {
    pub fn new() -> Self {
        Self { data: vec![], appending_list: vec![] }
    }

    // /// This is my own experimental method
    // pub fn append_encodable(&mut self, e: Box<dyn Encodable>) -> &mut Self { self }

    /* Mock Parity implementation */

    /// Boolean flag indicates whether the stream is still processing a list
    fn is_processing_list(&self) -> bool {
        self.appending_list.is_empty()
    }

    /// Finish appending to a current list
    fn finish_list(&mut self, pos: usize) {
        let data_len = self.data.len() - pos;
        let enc_vec = encode_length(data_len, LIST_OFFSET);
        let enc_len = enc_vec.len();
        self.data.extend(enc_vec);
        self.data[pos..].rotate_right(enc_len);
    }

    /// Increment the list of items appended. `items` indicates how many items appended.
    /// Caller ensure stream is processing list.
    fn list_appended(&mut self, items: usize) {
        let idx = self.appending_list.len() - 1;
        match self.appending_list.get_mut(idx) {
            None => {}
            Some((pos, pending_size)) => {
                if items > *pending_size { panic!("items cannot be more than size"); }
                *pending_size -= items;

                // the current list is done
                if *pending_size == 0 {
                    let p = *pos;
                    self.finish_list(p);
                    self.list_appended(1);
                }
            }
        }
    }

    pub fn begin_list(&mut self, len: usize) -> &mut Self {
        self.appending_list.push((self.data.len(), len));
        self
    }

    pub fn append<E: IntoIterator<Item=u8>>(&mut self, e: E) -> &mut Self {
        self.write_iter(e.into_iter());
        if self.is_processing_list() {
            self.list_appended(1);
        }
        self
    }

    pub fn write_iter<I: Iterator<Item=u8>>(&mut self, mut iter: I) {
        let len = match iter.size_hint() {
            (lo, Some(up)) if lo == up => lo,
            _ => {
                return self.write_iter(iter.collect::<Vec<_>>().into_iter());
            }
        };

        // refer to https://eth.wiki/fundamentals/rlp
        match len {
            0 => self.data.push(STR_OFFSET),
            1..55 => {
                let first = iter.next().expect("invalid iter size");
                if len == 1 && first < STR_OFFSET {
                    self.data.push(first);
                } else {
                    self.data.push(len as u8 + STR_OFFSET);
                    self.data.push(first);
                    self.data.extend(iter);
                }
            }
            _ => {
                let mut d = vec![];
                to_binary(len, &mut d);
                self.data.push(d.len() as u8 + STR_OFFSET + LEN_CUTOFF);
                self.data.extend(d);
                self.data.extend(iter);
            }
        }
    }

    pub fn get(&self) -> Vec<u8> {
        self.data.clone()
    }
}

fn to_binary(x: usize, data: &mut Vec<u8>) {
    if x == 0 {
        return;
    }
    to_binary(x / 256, data);
    data.push((x % 256) as u8);
}

fn encode_length(len: usize, offset: u8) -> Vec<u8> {
    match len {
        0..55 => vec![len as u8 + offset],
        _ => {
            let mut data = vec![];
            to_binary(len, &mut data);
            let mut out = vec![data.len() as u8 + offset + 55];
            out.extend(data);
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rlp::{RPLStream, STR_OFFSET, to_binary};

    #[test]
    fn to_binary_works() {
        let mut data = vec![];
        to_binary(260, &mut data);
        assert_eq!(data, vec![0x1, 0x4]);

        let mut data = vec![];
        to_binary(260454, &mut data);
        assert_eq!(data, vec![0x1, 0x4]);
    }

    #[test]
    fn write_iter_works() {
        let mut stream = RPLStream::new();
        stream.write_iter("cat".chars().as_str().bytes());
        assert_eq!(stream.get(), vec![0x83, 0x63, 0x61, 0x74]);

        let mut stream = RPLStream::new();
        stream.write_iter("".chars().as_str().bytes());
        assert_eq!(stream.get(), vec![STR_OFFSET]);

        let mut stream = RPLStream::new();
        let a = "Lorem ipsum dolor sit amet, consectetur adipisicing elit".chars().as_str().bytes();
        stream.write_iter(a.clone());
        let mut r = vec![0xb8, 0x38];
        r.extend(a);
        assert_eq!(stream.get(), r);
    }
}