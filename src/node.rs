//! Node module for the application.
//!
//! This module provides the `Node` structure and associated methods for managing
//! node information, including parsing node record strings and generating secret keys.

use crate::config::ETH_EXPECTED_PARTS_LEN;
use crate::errors::CustomError;

use secp256k1::{rand, SecretKey};

/// Represents a Node with its address, port, and secret key.
///
/// The `Node` struct provides methods to create a new node from a node record string,
/// parse the node record string, and retrieve the address, port, and secret key of the node.
pub struct Node {
    addr: String,
    port: u16,
    secret_key: SecretKey,
}

impl Node {
    pub fn new(node_record_str: &str) -> Result<Self, CustomError> {
        let (addr, port) = Node::parse_node_record(node_record_str)?;
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        Ok(Node {
            addr: addr.to_string(),
            port,
            secret_key,
        })
    }

    /// Parses a node record string to extract the address and port.
    ///
    /// This method splits the node record string to extract the address and port components,
    /// and checks if they match the expected format.
    ///
    /// # Arguments
    ///
    /// * `node_record_str` - The node record string to parse.
    ///
    /// # Returns
    ///
    /// A result containing a tuple with the address and port if successful, or a `CustomError` if parsing fails.
    pub fn parse_node_record(node_record_str: &str) -> Result<(&str, u16), CustomError> {
        let parts: Vec<&str> = node_record_str.split('@').collect();
        if parts.len() != ETH_EXPECTED_PARTS_LEN {
            return Err(CustomError::AddressPortParse);
        }

        let address_port: Vec<&str> = parts[1].split(':').collect();
        if address_port.len() != ETH_EXPECTED_PARTS_LEN {
            return Err(CustomError::AddressPortParse);
        }

        let addr = address_port[0];
        let port: u16 = address_port[1]
            .parse()
            .map_err(|_| CustomError::AddressPortParse)?;

        Ok((addr, port))
    }

    pub fn get_addr(&self) -> &str {
        &self.addr
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_secret_key(&self) -> &SecretKey {
        &self.secret_key
    }
}
