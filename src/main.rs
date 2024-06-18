use std::{str::FromStr, time::Duration};

use alloy_rlp::{Decodable, Encodable};
use clap::Parser;
use futures::{SinkExt, StreamExt};
use reth_ecies::stream::ECIESStream;
use reth_eth_wire::{HelloMessage, P2PMessage};
use reth_network_peers::{pk2id, NodeRecord};
use secp256k1::{rand, SecretKey, SECP256K1};
use thiserror::Error;
use tokio::net::TcpStream;

/// Struct for parsing command line arguments
#[derive(Parser, Debug)]
#[command(name = "Node Connector")]
#[command(about = "A CLI for connecting to an Ethereum node", long_about = None)]
struct Cli {
    /// NodeRecord string
    #[arg(long)]
    node_record: String,
}

#[derive(Debug, Error)]
enum CustomError {
    #[error("Failed to extract address and port from node record")]
    AddressPortParse,
    #[error("Failed to connect to the TCP stream: {0}")]
    TcpConnect(#[from] std::io::Error),
    #[error("Failed to create NodeRecord from string: {0}")]
    NodeRecordCreation(#[from] reth_network_peers::NodeRecordParseError),
    #[error("Failed to create ECIES stream")]
    ECIESStreamCreation,
    #[error("Failed to send message")]
    SendMessage,
    #[error("Failed to receive message")]
    ReceiveMessage,
    #[error("Failed to decode P2P message: {0}")]
    MessageDecode(#[from] alloy_rlp::Error),
}

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let cli = Cli::parse();
    let node_record_str = &cli.node_record;

    // Split the node_record_str to extract address and port
    let parts: Vec<&str> = node_record_str.split('@').collect();
    if parts.len() != 2 {
        return Err(CustomError::AddressPortParse);
    }

    let address_port: Vec<&str> = parts[1].split(':').collect();
    if address_port.len() != 2 {
        return Err(CustomError::AddressPortParse);
    }

    let addr = address_port[0];
    let port: u16 = address_port[1]
        .parse()
        .map_err(|_| CustomError::AddressPortParse)?;

    let secret_key = SecretKey::new(&mut rand::thread_rng());
    let outgoing = TcpStream::connect((addr, port))
        .await
        .map_err(|e| CustomError::TcpConnect(e))?;
    let node = NodeRecord::from_str(node_record_str).map_err(CustomError::NodeRecordCreation)?;
    let mut client_stream: ECIESStream<TcpStream> =
        ECIESStream::connect(outgoing, secret_key, node.id)
            .await
            .map_err(|_| CustomError::ECIESStreamCreation)?;

    let our_peer_id = pk2id(&secret_key.public_key(SECP256K1));
    let msg = HelloMessage::builder(our_peer_id).build().into_message();

    let hello = P2PMessage::Hello(msg);

    let mut hello_encoded = Vec::new();
    hello.encode(&mut hello_encoded);

    client_stream
        .send(hello_encoded.into())
        .await
        .map_err(|_| CustomError::SendMessage)?;

    let message_result = tokio::time::timeout(Duration::from_millis(1000), client_stream.next())
        .await
        .map_err(|_| CustomError::ReceiveMessage)?;

    let message = message_result.ok_or(CustomError::ReceiveMessage)??;

    let resp = P2PMessage::decode(&mut &message[..])?;

    println!("{:?}", resp);
    Ok(())
}
