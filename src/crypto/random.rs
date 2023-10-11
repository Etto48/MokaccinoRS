pub struct Random<const LEN: usize>
{   
    data: [u8; LEN]
}

impl<const LEN: usize> Random<LEN>
{
    pub fn new() -> Self
    {
        let mut data = [0u8; LEN];
        openssl::rand::rand_bytes(&mut data).unwrap();
        Self { data }
    }
}

impl<const LEN: usize> Into<[u8; LEN]> for Random<LEN>
{
    fn into(self) -> [u8; LEN] {
        self.data
    }
}

impl Into<u64> for Random<8>
{
    fn into(self) -> u64 {
        u64::from_le_bytes(self.data)
    }
}