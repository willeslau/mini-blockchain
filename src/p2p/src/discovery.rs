use common::H256;
use crate::db::Storage;
use crate::node::NodeEntry;

const ADDRESS_BYTES_SIZE: usize = 32; // Size of address type in bytes.

pub(crate) struct Discovery {

}

impl Discovery {
    /// Add a new node to discovery table. Pings the node.
    pub fn add_node(&mut self, e: NodeEntry) {}

    /// Add a list of nodes. Pings a few nodes each round
    pub fn add_node_list(&mut self, nodes: Vec<NodeEntry>) {}
}

/// The inner struct of Discovery, handles all the processing logic
pub(crate) struct DiscoveryInner<'a> {
    storage: &'a mut Storage,

}

impl <'a> DiscoveryInner<'a> {
    /// Add a new node to discovery table. Pings the node.
    fn add_node(&mut self, e: NodeEntry) {
    }

    /// Add a list of nodes. Pings a few nodes each round
    fn add_node_list(&mut self, nodes: Vec<NodeEntry>) {}

    fn distance(a: &H256, b: &H256) -> Option<usize> {
        let mut lz = 0;
        for i in 0..ADDRESS_BYTES_SIZE {
            let d: u8 = a[i] ^ b[i];
            if d == 0 {
                lz += 8;
            } else {
                lz += d.leading_zeros() as usize;
                return Some(ADDRESS_BYTES_SIZE * 8 - lz);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use common::H256;
    use crate::discovery::{ADDRESS_BYTES_SIZE, DiscoveryInner};

    #[test]
    fn distance_works() {
        let a= H256::from_slice(&[228, 104, 254, 227, 239, 33, 109, 25, 223, 95, 27, 195, 177, 52, 50, 204, 76, 30, 147, 218, 216, 159, 47, 146, 236, 13, 163, 128, 250, 160, 17, 192]);
        let b= H256::from_slice(&[228, 214, 227, 65, 84, 85, 107, 82, 209, 81, 68, 106, 172, 254, 164, 105, 92, 23, 184, 27, 10, 90, 228, 69, 143, 90, 18, 117, 49, 186, 231, 5]);

        let result = DiscoveryInner::distance(&a, &b);
        assert_eq!(result, Some(248));
    }
}
