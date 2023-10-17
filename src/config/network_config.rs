use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::{crypto::PrivateKey, network::LastingContactInfo};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig
{
    pub name: String,
    #[serde(default = "NetworkConfig::default_port")]
    pub port: u16,
    pub whitelist: Option<Vec<String>>,
    #[serde(default = "NetworkConfig::default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "NetworkConfig::default_ping_ms")]
    pub ping_ms: u64,
    #[serde(default = "NetworkConfig::default_timeout_strikes")]
    pub timeout_strikes: u16,
    pub private_key: PrivateKey,
    #[serde(default = "NetworkConfig::default_known_hosts")]
    pub known_hosts: HashMap<String,LastingContactInfo>,
}

impl NetworkConfig
{
    fn default_port() -> u16 { 4848 }
    fn default_timeout_ms() -> u64 { 100 }
    fn default_ping_ms() -> u64 { 1000 }
    fn default_timeout_strikes() -> u16 { 10 }
    fn default_known_hosts() -> HashMap<String,LastingContactInfo> { HashMap::new() }
}

impl Default for NetworkConfig
{
    fn default() -> Self {
        NetworkConfig{
            name: format!("Anonymous#{:x}", rand::random::<u64>()),
            port: NetworkConfig::default_port(),
            whitelist: None,
            timeout_ms: NetworkConfig::default_timeout_ms(),
            ping_ms: NetworkConfig::default_ping_ms(),
            timeout_strikes: NetworkConfig::default_timeout_strikes(),
            private_key: PrivateKey::new(),
            known_hosts: NetworkConfig::default_known_hosts(),
        }
    }
}