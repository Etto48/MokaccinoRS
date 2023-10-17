use std::net::SocketAddr;

use serializable::Serializable;

use crate::{crypto::PublicKey, config::Config};

use super::ConnectionList;

#[derive(Serializable, Clone, Debug, PartialEq)]
pub struct UserInfo
{
    name: String,
    address: Option<SocketAddr>,
    public_key: Option<PublicKey>,
}

impl UserInfo
{
    pub fn new(name: &str, connection_list: &ConnectionList, config: &Config) -> Self
    {
        let address = connection_list.get_address(name).copied();
        let public_key = if let Some(lasting_contact_info) = config.network.known_hosts.get(name)
        {
            Some(lasting_contact_info.crypto_info().public_key.clone())
        }
        else
        {
            None
        };
        Self { name: name.to_string(), address, public_key }
    }

    pub fn new_empty(name: &str) -> Self
    {
        Self { name: name.to_string(), address: None, public_key: None }
    }

    pub fn self_from_config(config: &Config) -> Self
    {
        Self
        {
            name: config.network.name.clone(),
            address: None,
            public_key: Some(config.network.private_key.public_key()),
        }
    }

    pub fn name(&self) -> &str
    {
        &self.name
    }

    pub fn address(&self) -> Option<SocketAddr>
    {
        self.address
    }

    pub fn public_key(&self) -> &Option<PublicKey>
    {
        &self.public_key
    }
}