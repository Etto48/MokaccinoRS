use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig
{
    pub name: String,
    pub port: u16,
    pub whitelist: Option<Vec<String>>,
    pub timeout_ms: u64,
    pub ping_ms: u64,
    pub timeout_strikes: u16
}

impl Default for NetworkConfig
{
    fn default() -> Self {
        NetworkConfig{
            name: "Anonymous".to_string(),
            port: 4848,
            whitelist: None,
            timeout_ms: 100,
            ping_ms: 1000,
            timeout_strikes: 10,
        }
    }
}