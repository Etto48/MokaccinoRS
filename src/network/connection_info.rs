use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConnectionInfo
{
    pub last_seen: Instant,
    pub strikes: u16,
    pub packet_loss: u32,
    pub latency: Duration
}

impl ConnectionInfo
{
    pub fn new() -> Self
    {
        Self { 
            last_seen: Instant::now(),
            strikes: 0,
            packet_loss: 0,
            latency: Duration::from_secs(0)
        }
    }

    pub fn add_strike(&mut self)
    {
        self.strikes += 1;
        self.last_seen = Instant::now();
    }

    pub fn reset_strikes(&mut self)
    {
       self.strikes = 0;
       self.last_seen = Instant::now();
    }
}