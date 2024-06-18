//! Configuration module for the application.

/// Represents the timeout value for the handshake in seconds.
pub const TIMEOUT: u64 = 10;

/// The expected number of parts in an Ethereum node record string.
/// This constant is used to validate the format of node record strings, which should
/// contain exactly two parts: the node identifier and the address with port, separated by '@'.
pub const ETH_EXPECTED_PARTS_LEN: usize = 2;
