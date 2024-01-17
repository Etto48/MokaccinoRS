use crate::config::defines;

use super::hash::hash;

pub struct ReceiveTransaction
{
    file_name: String,
    hash: Vec<u8>,
    next_byte: u64,
    data: Vec<u8>,
}

impl ReceiveTransaction {
    pub fn new(file_name: String, hash: Vec<u8>, size: u64) -> Self {
        //escape file_name
        let file_name = file_name.replace("/","_");
        let file_name = file_name.replace("\\","_");
        Self {
            file_name,
            hash,
            next_byte: 0,
            data: vec![0u8; size as usize],
        }
    }

    pub fn receive(&mut self, starting_byte: u64, data: Vec<u8>) {
        if self.next_byte < starting_byte + data.len() as u64 {
            self.data[starting_byte as usize..starting_byte as usize + data.len()].copy_from_slice(&data);
            self.next_byte = starting_byte + data.len() as u64;
        }
    }

    pub fn is_complete(&self) -> bool {
        self.next_byte == self.data.len() as u64 && self.check_hash()
    }

    fn check_hash(&self) -> bool {
        hash(&self.data) == self.hash
    }

    pub fn gen_ack(&self) -> u64 {
        self.next_byte
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        std::fs::write(format!("{}/{}",path,self.file_name), &self.data)?;
        Ok(())
    }
}

