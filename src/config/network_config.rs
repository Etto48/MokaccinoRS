use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::{crypto::PrivateKey, network::LastingContactInfo};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig
{
    pub name: String,
    pub port: u16,
    pub whitelist: Option<Vec<String>>,
    pub timeout_ms: u64,
    pub ping_ms: u64,
    pub timeout_strikes: u16,
    pub private_key: PrivateKey,
    pub known_hosts: HashMap<String,LastingContactInfo>,
}

impl Default for NetworkConfig
{
    fn default() -> Self {
        NetworkConfig{
            name: format!("Anonymous#{:x}", rand::random::<u64>()),
            port: 4848,
            whitelist: None,
            timeout_ms: 100,
            ping_ms: 1000,
            timeout_strikes: 10,
            private_key: PrivateKey::new(),
            known_hosts: HashMap::new(),
        }
    }
}