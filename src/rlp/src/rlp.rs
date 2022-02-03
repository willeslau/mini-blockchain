use crate::traits::Encodable;

const STR_OFFSET: u8 = 0x80;
const LIST_OFFSET: u8 = 0xc0;
const LEN_CUTOFF: u8 = 55;

/// The RPL encoding struct. Refer to https://eth.wiki/fundamentals/rlp.md for more info
#[derive(Default)]
pub struct RLPStream {
    data: Vec<u8>,
    /// The index of the list currently being inserted
    appending_list: Vec<(usize, usize)>,
}

impl RLPStream {
    pub fn new() -> Self {
        Self { data: vec![], appending_list: vec![] }
    }

    pub fn new_list(len: usize) -> Self {
        let mut r = Self { data: vec![], appending_list: vec![] };
        r.begin_list(len);
        r
    }

    /* Mock Parity implementation */

    /// Boolean flag indicates whether the stream is still processing a list
    fn is_processing_list(&self) -> bool {
        !self.appending_list.is_empty()
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
        if !self.is_processing_list() { return; }
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
                    self.appending_list.pop();
                    self.list_appended(1);
                }
            }
        }
    }

    /// Marks the beginning of a list.
    pub fn begin_list(&mut self, len: usize) -> &mut Self {
        match len {
            0 => {
                self.data.push(LIST_OFFSET);
                self.list_appended(1);
            },
            _ => self.appending_list.push((self.data.len(), len)),
        }
        self
    }

    /// Append an encodable struct to the stream
    /// ```
    /// use rlp::RLPStream;
    /// let mut stream = RLPStream::new();
    /// stream.append(&"cat");
    /// assert_eq!(stream.out(), vec![0x83, 0x63, 0x61, 0x74]);
    /// ```
    pub fn append<E: Encodable>(&mut self, e: &E) -> &mut Self {
        e.encode(self);
        if self.is_processing_list() {
            self.list_appended(1);
        }
        self
    }

    /// Appends null to the end of stream, chainable.
    /// ```
    /// use rlp::RLPStream;
    /// let mut stream = RLPStream::new_list(2);
    /// stream.append_empty_data().append_empty_data();
    /// let out = stream.out();
    /// assert_eq!(out, vec![0xc2, 0x80, 0x80]);
    /// ```
    pub fn append_empty(&mut self) -> &mut Self {
        self.data.push(0x80);
        self.list_appended(1);
        self
    }

    pub fn append_raw(&mut self, raw: &[u8]) -> &mut Self {
        self.data.extend_from_slice(raw);
        self.list_appended(1);
        self
    }

    /// Write iterator into the stream. Should be invoked only by Encodable
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

    pub fn out(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn as_bytes(&self) -> &[u8] { self.data.as_slice() }
}

impl From<RLPStream> for Vec<u8> {
    fn from(r: RLPStream) -> Self {
        r.data
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
    use crate::rlp::{RLPStream, STR_OFFSET, to_binary};

    #[test]
    fn to_binary_works() {
        let mut data = vec![];
        to_binary(260, &mut data);
        assert_eq!(data, vec![0x1, 0x4]);

        let mut data = vec![];
        to_binary(260454, &mut data);
        assert_eq!(data, vec![3, 249, 102]);
    }

    #[test]
    fn write_iter_works() {
        let mut stream = RLPStream::new();
        stream.write_iter("cat".chars().as_str().bytes());
        assert_eq!(stream.out(), vec![0x83, 0x63, 0x61, 0x74]);

        let mut stream = RLPStream::new();
        stream.write_iter("".chars().as_str().bytes());
        assert_eq!(stream.out(), vec![STR_OFFSET]);

        let mut stream = RLPStream::new();
        let a = "Lorem ipsum dolor sit amet, consectetur adipisicing elit".chars().as_str().bytes();
        stream.write_iter(a.clone());
        let mut r = vec![0xb8, 0x38];
        r.extend(a);
        assert_eq!(stream.out(), r);
    }

    #[test]
    fn append_item_works() {
        // let mut stream = RPLStream::new();
        // stream.append("\x00");
        // assert_eq!(stream.get(), vec![ 0x80 ]);

        let mut stream = RLPStream::new();
        stream.append(&"cat");
        assert_eq!(stream.out(), vec![0x83, 0x63, 0x61, 0x74]);

        let mut stream = RLPStream::new();
        stream.append(&"dog");
        assert_eq!(stream.out(), vec![0x83, 0x64, 0x6F, 0x67]);

        let mut stream = RLPStream::new();
        stream.append(&"");
        assert_eq!(stream.out(), vec![STR_OFFSET]);

        let mut stream = RLPStream::new();
        let a = "Lorem ipsum dolor sit amet, consectetur adipisicing elit";
        stream.append(&a.clone());
        let mut r = vec![0xb8, 0x38];
        r.extend(a.bytes());
        assert_eq!(stream.out(), r);
    }

    #[test]
    fn append_list_works() {
        let stream = RLPStream::new_list(0);
        assert_eq!(stream.out(), vec![0xc0]);

        let mut stream = RLPStream::new_list(2);
        stream.append(&"cat").append(&"dog");
        assert_eq!(stream.out(), vec![0xc8, 0x83, 0x63, 0x61, 0x74, 0x83, 0x64, 0x6F, 0x67 ]);

        // [ [], [[]], [ [], [[]] ] ]
        let mut stream = RLPStream::new_list(3);
        stream.begin_list(0); // []
        stream.begin_list(1).begin_list(0); // [[]]
        stream.begin_list(2).begin_list(0).begin_list(1).begin_list(0); // [[], [[]]]
        assert_eq!(stream.out(), vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0 ]);
    }


    #[test]
    fn append_empty_works() {
        let mut stream = RLPStream::new_list(2);
        stream.append_empty().append_empty();
        let out = stream.out();
        assert_eq!(out, vec![0xc2, 0x80, 0x80]);
    }
}