//! Main module for the application.
//!
//! This module initializes the application, loads configuration settings, sets up logging,
//! parses command-line arguments, and performs the main operations such as creating a client
//! stream and performing a handshake with a remote Ethereum node.

use clap::Parser;
use env_logger::Env;
use log::{error, info};

use p2phandshake::network::{create_client_stream, handshake};
use p2phandshake::{cli::Cli, errors::CustomError, node::Node};

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    let node_record_str = &cli.node_record;

    match Node::new(node_record_str) {
        Ok(node) => {
            let addr = node.get_addr();
            let port = node.get_port();
            let secret_key = node.get_secret_key();

            match create_client_stream(addr, port, node_record_str, secret_key).await {
                Ok(mut client_stream) => {
                    if let Err(e) = handshake(&mut client_stream, secret_key).await {
                        error!("Handshake failed: {:?}", e);
                        return Err(e);
                    }
                }
                Err(e) => {
                    error!("Failed to create client stream: {:?}", e);
                    return Err(e);
                }
            }
        }
        Err(e) => {
            error!("Failed to initialize node: {:?}", e);
            return Err(e);
        }
    }

    info!("Program completed successfully.");
    Ok(())
}
