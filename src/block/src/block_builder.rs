// use transaction::{Executable, Transaction};
// use crate::{Block, Header};
//
// pub struct SimpleBlockBuilder {
//     executables: Vec<Transaction>,
// }
//
// impl SimpleBlockBuilder {
//     pub fn new() -> Self {
//         SimpleBlockBuilder{
//             executables: vec![],
//         }
//     }
//
//     pub fn add_executable(mut self, executable: ) -> Self {
//         self.executables.push(executable);
//         self
//     }
//
//     pub fn build(self) -> SimpleBlockBuilder {
//         Block::new(SimpleBlockBuilder::new(), self.executables)
//     }
// }