use std::{str::FromStr, time::Duration};

use alloy_rlp::{Decodable, Encodable};
use futures::{SinkExt, StreamExt};
use reth_ecies::stream::ECIESStream;
use reth_eth_wire::{HelloMessage, P2PMessage};
use reth_network_peers::{pk2id, NodeRecord};
use secp256k1::{rand, SecretKey, SECP256K1};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let secret_key = SecretKey::new(&mut rand::thread_rng());
    let outgoing = TcpStream::connect(("86.81.28.194", 30304)).await.unwrap();
    let node = NodeRecord::from_str("enode://7dcc9ea5437e5ef2fc681292cdb52d9f539ef11cb8404ed1b606bb4aa15199bf461f27e8cfb73d4357a97b3a33e9624b84ad04ddd9cbe136620de38b7c6b3238@86.81.28.194:30304
    ").unwrap();
    let mut client_stream: ECIESStream<TcpStream> =
        ECIESStream::connect(outgoing, secret_key, node.id)
            .await
            .unwrap();

    let our_peer_id = pk2id(&secret_key.public_key(SECP256K1));
    let msg = HelloMessage::builder(our_peer_id).build().into_message();

    let hello = P2PMessage::Hello(msg);

    let mut hello_encoded = Vec::new();
    hello.encode(&mut hello_encoded);

    client_stream.send(hello_encoded.into()).await.unwrap();

    let message = tokio::time::timeout(Duration::from_millis(1000), client_stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    let resp = P2PMessage::decode(&mut &message[..]).unwrap();
    println!("{:?}", resp);
}
