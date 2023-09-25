use std::net::SocketAddr;

pub enum VoiceRequest
{
    StartTransmission(SocketAddr),
    StopTransmission
}