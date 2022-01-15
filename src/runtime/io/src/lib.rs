mod worker;
mod service;
mod handler;
mod error;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    // #[test]
    // fn slab_works() {
    //     let mut s = slab::Slab::new();
    //     let i = s.insert(123);
    //     let j = s.insert(124);
    //     println!("{}, {}", i, j);
    // }
}
