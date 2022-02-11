# Introduction
The implementation of Ethereum P2P is based on Kademlia (at `https://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf`).
Ethereum has got some customization on top of the original paper, but to understand fully, we should examine Kademlia first.

# Kademlia
Participating computers each have a node ID in the 160-bit key space. <key,value> pairs are stored on nodes with IDs 
“close” to the key for some notion of closeness. This distance metric is XOR of the 160 bits node id.
Finally, a node-ID-based routing algorithm lets anyone locate servers near a destination key.

# Command
```shell
cargo build
RUST_LOG=debug cargo run --package p2p --bin discovery-basic
```