#![feature(exclusive_range_pattern)]

mod traits;
mod rlp;
mod error;

pub use crate::rlp::RPLStream;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
