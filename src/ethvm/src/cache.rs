use std::collections::HashSet;
use crate::error::Error;
use crate::instructions::Instruction;

pub(crate) struct JumpCache {
    jump_location: HashSet<usize>
}

impl JumpCache {
    pub fn new(code: &[u8]) -> Self {
        Self {
            jump_location: Self::find_jump_destination(code)
        }
    }

    pub fn valid_jump_dest(&self, dest: usize) -> Result<(), Error> {
        self.jump_location.contains(&dest).then(|| ()).ok_or(Error::InvalidJump)
    }

    fn find_jump_destination(code: &[u8]) -> HashSet<usize> {
        let mut set = HashSet::new();
        for pos in 0..code.len() {
            let instruction = Instruction::from_u8(code[pos]).expect("invalid instruction code.qed");
            match instruction {
                Instruction::JUMPDEST => {
                    set.insert(pos);
                },
                _ => {},
            };
        }
        set
    }
}