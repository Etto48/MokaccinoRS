use crate::config::defines;

pub fn create(port: u16) -> Result<std::net::UdpSocket,String>
{
    if let Ok(socket) = std::net::UdpSocket::bind(format!("0.0.0.0:{port}"))
    {
        socket.set_read_timeout(Some(defines::THREAD_QUEUE_TIMEOUT)).map_err(|e| e.to_string())?;
        Ok(socket)
    }
    else
    {
        Err(format!("Failed to bind to port {port}"))
    }
}