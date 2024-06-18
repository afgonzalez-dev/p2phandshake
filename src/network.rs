//! Network module for the application.
//!
//! This module provides functions for network operations such as creating client streams,
//! sending and receiving messages, and performing handshake operations with remote Ethereum nodes.

use crate::{config::TIMEOUT, errors::CustomError};

use alloy_rlp::{Decodable, Encodable};
use futures::{SinkExt, StreamExt};
use log::{debug, error, info};
use reth_ecies::stream::ECIESStream;
use reth_eth_wire::{DisconnectReason, HelloMessage, P2PMessage};
use reth_network_peers::{pk2id, NodeRecord};
use secp256k1::{SecretKey, SECP256K1};
use std::{str::FromStr, time::Duration};
use tokio::net::TcpStream;

/// Creates a client stream for connecting to a remote Ethereum node.
///
/// This function attempts to connect to the provided address and port,
/// and then establishes an ECIES stream for encrypted communication.
///
/// # Arguments
///
/// * `addr` - The address of the remote Ethereum node.
/// * `port` - The port of the remote Ethereum node.
/// * `node_record_str` - The node record string for the remote Ethereum node.
/// * `secret_key` - The secret key used for establishing the ECIES stream.
///
/// # Returns
///
/// A result containing the ECIES stream if successful, or a `CustomError` if the connection fails.
pub async fn create_client_stream(
    addr: &str,
    port: u16,
    node_record_str: &str,
    secret_key: &SecretKey,
) -> Result<ECIESStream<TcpStream>, CustomError> {
    let outgoing = tokio::time::timeout(
        Duration::from_secs(TIMEOUT),
        TcpStream::connect((addr, port)),
    )
    .await
    .map_err(|_| CustomError::TcpConnectTimeOut("Connection timed out".to_string()))??;

    let node = NodeRecord::from_str(node_record_str).map_err(CustomError::NodeRecordCreation)?;
    ECIESStream::connect_with_timeout(outgoing, *secret_key, node.id, Duration::from_secs(1))
        .await
        .map_err(|_| CustomError::ECIESStreamCreation)
}

/// Sends a message over the provided client stream.
///
/// This function encodes the given P2P message and sends it over the client stream.
///
/// # Arguments
///
/// * `client_stream` - The ECIES client stream.
/// * `message` - The P2P message to send.
///
/// # Returns
///
/// A result indicating success or failure, with a `CustomError` if the send operation fails.
pub async fn send_message(
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

/// Sends a Hello message over the provided client stream.
///
/// This function constructs a Hello message and sends it over the client stream.
///
/// # Arguments
///
/// * `client_stream` - The ECIES client stream.
/// * `secret_key` - The secret key used for constructing the Hello message.
///
/// # Returns
///
/// A result indicating success or failure, with a `CustomError` if the send operation fails.
pub async fn send_hello_message(
    client_stream: &mut ECIESStream<TcpStream>,
    secret_key: &SecretKey,
) -> Result<(), CustomError> {
    let our_peer_id = pk2id(&secret_key.public_key(SECP256K1));
    let msg = HelloMessage::builder(our_peer_id).build().into_message();

    let hello = P2PMessage::Hello(msg);
    send_message(client_stream, hello).await
}

/// Sends a Disconnect message over the provided client stream.
///
/// This function constructs a Disconnect message and sends it over the client stream.
///
/// # Arguments
///
/// * `client_stream` - The ECIES client stream.
///
/// # Returns
///
/// A result indicating success or failure, with a `CustomError` if the send operation fails.
pub async fn send_disconnect_message(
    client_stream: &mut ECIESStream<TcpStream>,
) -> Result<(), CustomError> {
    let disconnect = P2PMessage::Disconnect(DisconnectReason::ClientQuitting);
    send_message(client_stream, disconnect).await
}

/// Receives a P2P message from the provided client stream.
///
/// This function waits for a message from the client stream and decodes it.
///
/// # Arguments
///
/// * `client_stream` - The ECIES client stream.
///
/// # Returns
///
/// A result containing the decoded P2P message if successful, or a `CustomError` if the receive operation fails.
pub async fn receive_p2p_message(
    client_stream: &mut ECIESStream<TcpStream>,
) -> Result<P2PMessage, CustomError> {
    let message_result = tokio::time::timeout(Duration::from_secs(TIMEOUT), client_stream.next())
        .await
        .map_err(|_| CustomError::ReceiveMessage)?;

    let message = message_result.ok_or(CustomError::ReceiveMessage)?.unwrap();

    let resp = P2PMessage::decode(&mut &message[..])?;
    Ok(resp)
}

/// Performs a handshake with the remote Ethereum node.
///
/// This function sends a Hello message, waits for a response, and then sends a Disconnect message.
///
/// # Arguments
///
/// * `client_stream` - The ECIES client stream.
/// * `secret_key` - The secret key used for constructing the Hello message.
///
/// # Returns
///
/// A result indicating success or failure, with a `CustomError` if any part of the handshake fails.
pub async fn handshake(
    client_stream: &mut ECIESStream<TcpStream>,
    secret_key: &SecretKey,
) -> Result<(), CustomError> {
    info!("Initiating handshake...");

    info!("Sending Hello message...");
    if let Err(e) = send_hello_message(client_stream, secret_key).await {
        error!("Failed to send Hello message: {:?}", e);
        return Err(e);
    }
    debug!("Hello message sent.");

    info!("Waiting for P2P message...");
    match receive_p2p_message(client_stream).await {
        Ok(response) => {
            info!("Received P2P message: {:?}", response);
        }
        Err(e) => {
            error!("Failed to receive P2P message: {:?}", e);
            return Err(e);
        }
    }

    info!("Sending Disconnect message...");
    if let Err(e) = send_disconnect_message(client_stream).await {
        error!("Failed to send Disconnect message: {:?}", e);
        return Err(e);
    }
    debug!("Disconnect message sent.");

    info!("Handshake completed successfully.");
    Ok(())
}
