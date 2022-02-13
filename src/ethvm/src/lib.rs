mod cost;
mod error;
mod gas;
mod instructions;
mod interpreter;
mod memory;
mod stack;
mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
