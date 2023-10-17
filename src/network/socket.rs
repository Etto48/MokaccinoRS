use crate::config::defines;

pub fn create(port: u16) -> Result<std::net::UdpSocket,String>
{
    let host_addr = std::net::SocketAddr::new(defines::HOST, port);
    if let Ok(socket) = std::net::UdpSocket::bind(host_addr)
    {
        socket.set_read_timeout(Some(defines::THREAD_QUEUE_TIMEOUT)).map_err(|e| e.to_string())?;
        Ok(socket)
    }
    else
    {
        Err(format!("Failed to bind to port {port}"))
    }
}