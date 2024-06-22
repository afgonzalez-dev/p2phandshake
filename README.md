# P2P Node Handshake

This project demonstrates a P2P handshake with an Ethereum node using Rust. It connects to a specified node, performs a handshake by sending a `Hello` message, waits for a response, and then disconnects.

## Requirements
- Rust 1.76 or later

## Setup

### 1. Clone the repository
git clone git@github.com:afgonzalez-dev/p2phandshake.git

cd p2phandshake
### 2. cargo build

# How to run
Command-line Arguments:

--node-record: The node record string of the Ethereum node to connect to.

``` cargo run -- --node-record "enode://7dcc9ea5437e5ef2fc681292cdb52d9f539ef11cb8404ed1b606bb4aa15199bf461f27e8cfb73d4357a97b3a33e9624b84ad04ddd9cbe136620de38b7c6b3238@86.81.28.194:30304" ```

# Logging (DEBUG)

```
RUST_LOG=debug cargo run -- --node-record "enode://7dcc9ea5437e5ef2fc681292cdb52d9f539ef11cb8404ed1b606bb4aa15199bf461f27e8cfb73d4357a97b3a33e9624b84ad04ddd9cbe136620de38b7c6b3238@86.81.28.194:30304"
```

# Project Structure

```
src/
├── config.rs      # Configuration module
├── errors.rs      # Custom errors module
├── main.rs        # Entry point of the application
├── network.rs     # Network operations module
├── node.rs        # Node structure and methods module
├── cli.rs         # Command-line interface parsing module
├── lib.rs         # Main library module
└── tests/
    └── mod.rs     # Integration tests
```

## License
This project is licensed under the MIT License. See the LICENSE file for details.

