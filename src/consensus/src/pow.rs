use block::{SimpleBlock, Block, Header};
use crate::Consensus;
use sha2::{Sha256, Digest};
use primitives::StringSerializable;

pub struct ProofOfWork {
    block: SimpleBlock,
    nonce: u32,
    max_nonce: u32,
    prefix: Box<str>,
}

impl ProofOfWork {
    fn new(block: SimpleBlock, prefix: Box<str>) -> ProofOfWork {
        ProofOfWork{
            block,
            nonce: 0,
            max_nonce: u32::MAX,
            prefix,
        }
    }
}

impl Consensus for ProofOfWork {
    type Block = SimpleBlock;

    fn seal(&mut self) -> bool {
        while !self.validate() {
            if self.nonce == self.max_nonce {
                log::warn!("max nonce reached, cannot seal block");
                return false;
            }
            self.nonce += 1
        }
        log::debug!(
            "POW sealed, found nonce for block {} to be {}",
            self.block.header().block_number(),
            self.nonce
        );

        true
    }

    fn validate(&self) -> bool {
        let executables = self.block.executables();
        let mut concat = String::new();

        // join the executables
        for e in executables {
            let tmp = concat;
            concat = [tmp.clone(), (*e.serialize()).to_string()].join("").clone();
        }

        // join the nonce
        let concat = &[concat, self.nonce.to_string()].join("");

        let mut hasher = Sha256::new();
        hasher.update(concat);
        let result = hasher.finalize();

        let base64_encoded = base64::encode(&result);
        base64_encoded.starts_with(&*self.prefix)
    }

    fn block(&self) -> Self::Block {
        self.block.clone()
    }
}


#[cfg(test)]
mod tests {
    use crate::pow::ProofOfWork;
    use crate::Consensus;
    use block::{SimpleBlock, Block, SimpleHeader, Header};
    use transaction::MockTransaction;

    #[test]
    fn validate_works() {
        let simple_block = SimpleBlock::new(
            SimpleHeader::new(),
            vec![
                MockTransaction::new("this is a test".parse().unwrap()),
                MockTransaction::new("this is a test".parse().unwrap()),
                MockTransaction::new("this is a test".parse().unwrap()),
            ]
        );

        let mut pow = ProofOfWork::new(
            simple_block,
            Box::from("0000"),
        );
        pow.seal();
    }
}