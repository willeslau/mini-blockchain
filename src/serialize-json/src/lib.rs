pub mod vm;
pub mod state;
mod bytes;
mod hash;
mod maybe;
mod uint;
mod test;
mod transaction;
mod blockchain;
mod local_tests;
mod spec;
mod trie;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
