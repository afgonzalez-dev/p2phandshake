use alloy_rlp;
use reth_network_peers;
use std::io;
use thiserror::Error;
use tokio::time::error::Elapsed;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("Failed to extract address and port from node record")]
    AddressPortParse,
    #[error("Failed to connect to the TCP stream: {0}")]
    TcpConnectTimeOut(String),
    #[error("Failed to connect to the TCP stream: {0}")]
    TcpConnect(#[from] io::Error),
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

impl From<Elapsed> for CustomError {
    fn from(_: Elapsed) -> Self {
        CustomError::TcpConnectTimeOut("Timeout".to_string())
    }
}

impl PartialEq for CustomError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (CustomError::AddressPortParse, CustomError::AddressPortParse)
                | (CustomError::TcpConnect(_), CustomError::TcpConnect(_))
                | (
                    CustomError::NodeRecordCreation(_),
                    CustomError::NodeRecordCreation(_)
                )
                | (
                    CustomError::ECIESStreamCreation,
                    CustomError::ECIESStreamCreation
                )
                | (CustomError::SendMessage, CustomError::SendMessage)
                | (CustomError::ReceiveMessage, CustomError::ReceiveMessage)
                | (CustomError::MessageDecode(_), CustomError::MessageDecode(_))
        )
    }
}
