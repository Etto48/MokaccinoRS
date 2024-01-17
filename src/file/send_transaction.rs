use super::hash;

pub struct SendTransaction
{
    file_name: String,
    hash: Vec<u8>,
    last_acked: u64,
    last_sent: u64,
    data: Vec<u8>,
}

impl SendTransaction
{
    pub fn new(file_name: &str, data: Vec<u8>) -> Self
    {
        let hash = hash::hash(&data);
        Self
        {
            file_name: file_name.to_string(),
            hash,
            last_acked: 0,
            last_sent: 0,
            data,
        }
    }

    pub fn from_file(path: &str) -> Self
    {
        let file_name = path.split("/").last().unwrap();
        let data = std::fs::read(path).unwrap();
        Self::new(file_name, data)
    }
}