use std::{net::SocketAddr, collections::HashMap};

use super::connection_info::ConnectionInfo;

pub struct ConnectionList
{
    names_to_addresses: HashMap<String,SocketAddr>,
    addresses_to_names: HashMap<SocketAddr,String>,
    address_to_info: HashMap<SocketAddr,ConnectionInfo>,
}

impl ConnectionList
{
    pub fn new() -> Self
    {
        Self {
            names_to_addresses: HashMap::new(),
            addresses_to_names: HashMap::new(),
            address_to_info: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: &str, address: SocketAddr)
    {
        self.names_to_addresses.insert(name.to_string(),address);
        self.addresses_to_names.insert(address,name.to_string());
        self.address_to_info.insert(address,ConnectionInfo::new());
    }

    pub fn remove_with_name(&mut self, name: &str)
    {
        if let Some(address) = self.names_to_addresses.remove(name)
        {
            self.addresses_to_names.remove(&address);
            self.address_to_info.remove(&address);
        }
    }

    pub fn remove_with_address(&mut self, address: &SocketAddr)
    {
        if let Some(name) = self.addresses_to_names.remove(address)
        {
            self.names_to_addresses.remove(&name);
            self.address_to_info.remove(address);
        }
    }

    pub fn get_address(&self, name: &str) -> Option<&SocketAddr>
    {
        self.names_to_addresses.get(name)
    }

    pub fn get_name(&self, address: &SocketAddr) -> Option<&str>
    {
        self.addresses_to_names.get(address).map(|s|s.as_str())
    }

    pub fn get_info_from_addr(&self, address: &SocketAddr) -> Option<&ConnectionInfo>
    {
        self.address_to_info.get(address)
    }

    pub fn get_info_from_name(&self, name: &str) -> Option<&ConnectionInfo>
    {
        if let Some(address) = self.get_address(name)
        {
            self.get_info_from_addr(&address)
        }
        else
        {
            None
        }
    }

    pub fn get_address_mut(&mut self, name: &str) -> Option<&mut SocketAddr>
    {
        self.names_to_addresses.get_mut(name)
    }

    pub fn get_name_mut(&mut self, address: &SocketAddr) -> Option<&mut String>
    {
        self.addresses_to_names.get_mut(address)
    }

    pub fn get_info_from_addr_mut(&mut self, address: &SocketAddr) -> Option<&mut ConnectionInfo>
    {
        self.address_to_info.get_mut(address)
    }

    pub fn get_info_from_name_mut(&mut self, name: &str) -> Option<&mut ConnectionInfo>
    {
        if let Some(address) = self.get_address(name)
        {
            let address = address.clone();
            self.get_info_from_addr_mut(&address)
        }
        else
        {
            None
        }
    }

    pub fn get_names(&self) -> Vec<String>
    {
        self.names_to_addresses.keys().cloned().collect()
    }

    pub fn get_addresses(&self) -> Vec<SocketAddr>
    {
        self.addresses_to_names.keys().cloned().collect()
    }

    pub fn get_infos(&self) -> Vec<(SocketAddr,ConnectionInfo)>
    {
        self.address_to_info.iter().map(|(address,info)|(*address,*info)).collect()
    }
}