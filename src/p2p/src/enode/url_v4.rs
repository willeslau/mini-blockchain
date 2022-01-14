use common::{Hasher, KeccakHasher, Public};
use crate::enode::node::NodeId;

/// pubkey_to_idv4 derives the v4 node address from the given public key.
pub(crate) fn pubkey_to_idv4(key: &Public) -> NodeId {
    KeccakHasher::hash(key)
}