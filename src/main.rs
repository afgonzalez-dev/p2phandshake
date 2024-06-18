use std::{str::FromStr, time::Duration};

use alloy_rlp::{Decodable, Encodable};
use clap::Parser;
use futures::{SinkExt, StreamExt};
use reth_ecies::stream::ECIESStream;
use reth_eth_wire::{DisconnectReason, HelloMessage, P2PMessage};
use reth_network_peers::{pk2id, NodeRecord};
use secp256k1::{rand, SecretKey, SECP256K1};
use tokio::net::TcpStream;

mod errors;
use errors::CustomError;

/// Struct for parsing command line arguments
#[derive(Parser, Debug)]
#[command(name = "Node Connector")]
#[command(about = "A CLI for connecting to an Ethereum node", long_about = None)]
struct Cli {
    /// NodeRecord string
    #[arg(long)]
    node_record: String,
}

fn parse_node_record(node_record_str: &str) -> Result<(&str, u16), CustomError> {
    const ETH_EXPECTED_PARTS_LEN: usize = 2;

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

async fn create_client_stream(
    addr: &str,
    port: u16,
    node_record_str: &str,
    secret_key: &SecretKey,
) -> Result<ECIESStream<TcpStream>, CustomError> {
    let outgoing = TcpStream::connect((addr, port))
        .await
        .map_err(CustomError::TcpConnect)?;
    let node = NodeRecord::from_str(node_record_str).map_err(CustomError::NodeRecordCreation)?;
    ECIESStream::connect(outgoing, *secret_key, node.id)
        .await
        .map_err(|_| CustomError::ECIESStreamCreation)
}

async fn send_message(
    client_stream: &mut ECIESStream<TcpStream>,
    message: P2PMessage,
) -> Result<(), CustomError> {
    let mut encoded_msg = Vec::new();
    message.encode(&mut encoded_msg);

    client_stream
        .send(encoded_msg.into())
        .await
        .map_err(|_| CustomError::SendMessage)
}

async fn send_hello_message(
    client_stream: &mut ECIESStream<TcpStream>,
    secret_key: &SecretKey,
) -> Result<(), CustomError> {
    let our_peer_id = pk2id(&secret_key.public_key(SECP256K1));
    let msg = HelloMessage::builder(our_peer_id).build().into_message();

    let hello = P2PMessage::Hello(msg);
    send_message(client_stream, hello).await
}

async fn send_disconnect_message(
    client_stream: &mut ECIESStream<TcpStream>,
) -> Result<(), CustomError> {
    let disconnect = P2PMessage::Disconnect(DisconnectReason::ClientQuitting);
    send_message(client_stream, disconnect).await
}

async fn receive_p2p_message(
    client_stream: &mut ECIESStream<TcpStream>,
) -> Result<P2PMessage, CustomError> {
    let message_result = tokio::time::timeout(Duration::from_millis(1000), client_stream.next())
        .await
        .map_err(|_| CustomError::ReceiveMessage)?;

    let message = message_result.ok_or(CustomError::ReceiveMessage)??;

    let resp = P2PMessage::decode(&mut &message[..])?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), CustomError> {
    let cli = Cli::parse();
    let node_record_str = &cli.node_record;

    let (addr, port) = parse_node_record(node_record_str)?;

    let secret_key = SecretKey::new(&mut rand::thread_rng());
    let mut client_stream = create_client_stream(addr, port, node_record_str, &secret_key).await?;

    send_hello_message(&mut client_stream, &secret_key).await?;
    let resp = receive_p2p_message(&mut client_stream).await?;

    send_disconnect_message(&mut client_stream).await?;

    Ok(())
}
