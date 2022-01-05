# mini-blockchain
A mini blockchain for studying purposes and rust. 
Based on the implementation of `go-ethereum`.

## Build

## Run tests with coverage
To run tests and check coverage, run the below:
```bash
rustup component add llvm-tools-preview
export RUSTFLAGS="-Zinstrument-coverage"
cargo build
export LLVM_PROFILE_FILE="your_name-%p-%m.profraw"
cargo test
```

Reading:
* Solidity Programming Essentials
* Mastering Ethereum
* Programming Bitcoin
* Ethereum Cookbook
* Mastering Ethereum
* Building Ethereum DApps
* Grokking Bitcoin