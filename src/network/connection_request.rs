use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub enum ConnectionRequest
{
    Connect(SocketAddr),
    Disconnect(String),
}