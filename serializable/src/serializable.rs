use std::time::SystemTime;

pub trait Serializable: Sized
{
    /// Returns the serialized object as a vector of bytes
    fn serialize(&self) -> Vec<u8>;
    /// Returns the deserialized object and the number of bytes read
    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)>;
}

impl Serializable for std::net::SocketAddr
{
    fn serialize(&self) -> Vec<u8> {
        match self {
            std::net::SocketAddr::V4(addr) => {
                let mut vec = Vec::new();
                vec.push(0);
                vec.extend_from_slice(&addr.ip().octets());
                vec.extend_from_slice(&addr.port().to_be_bytes());
                vec
            },
            std::net::SocketAddr::V6(addr) => {
                let mut vec = Vec::new();
                vec.push(1);
                vec.extend_from_slice(&addr.ip().octets());
                vec.extend_from_slice(&addr.port().to_be_bytes());
                vec
            }   
        }
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 1
        {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"));
        }
        else 
        {
            match data[0] {
                0 => {
                    if data.len() < 7
                    {
                        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
                    }
                    else
                    {
                        let ip = std::net::Ipv4Addr::new(data[1], data[2], data[3], data[4]);
                        let port = u16::from_be_bytes([data[5], data[6]]);
                        let ret = std::net::SocketAddr::V4(std::net::SocketAddrV4::new(ip, port));
                        Ok((ret,7))
                    }
                },
                1 => {
                    if data.len() < 19
                    {
                        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
                    }
                    else
                    {
                        let ip = std::net::Ipv6Addr::new(
                            u16::from_be_bytes([data[1], data[2]]),
                            u16::from_be_bytes([data[3], data[4]]),
                            u16::from_be_bytes([data[5], data[6]]),
                            u16::from_be_bytes([data[7], data[8]]),
                            u16::from_be_bytes([data[9], data[10]]),
                            u16::from_be_bytes([data[11], data[12]]),
                            u16::from_be_bytes([data[13], data[14]]),
                            u16::from_be_bytes([data[15], data[16]])
                        );
                        let port = u16::from_be_bytes([data[17], data[18]]);
                        let ret = std::net::SocketAddr::V6(std::net::SocketAddrV6::new(ip, port, 0, 0));
                        Ok((ret,19))
                    }
                },
                _ => {
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid address type"))
                }
            }    
        }
    }
}

impl Serializable for String
{
    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend_from_slice(&mut (self.len() as u32).to_be_bytes());
        vec.extend_from_slice(self.as_bytes());
        vec
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 4
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else
        {
            let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            if data.len() < (len + 4) as usize
            {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
            }
            else
            {
                let mut vec = Vec::new();
                vec.extend_from_slice(&data[4..(len + 4) as usize]);
                let ret = String::from_utf8(vec).map_err(|e|std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid utf8 string format {e}")))?;
                Ok((ret, (len + 4) as usize))
            }
        }
    }
}

impl <T: Serializable> Serializable for Vec<T>
{
    fn serialize(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.extend((self.len() as u32).to_be_bytes());
        for item in self
        {
            ret.extend(item.serialize());
        }
        ret
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 4
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else
        {
            let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            let mut ret = Vec::new();
            let mut read = 4;
            for _ in 0..len
            {
                let (item, item_len) = T::deserialize(&data[read..])?;
                ret.push(item);
                read += item_len;
            }
            Ok((ret, read))
        }
    }
}

impl Serializable for u128
{
    fn serialize(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 16
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else
        {
            let ret = u128::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]);
            Ok((ret, 16))
        }
    }
}

impl Serializable for u64
{
    fn serialize(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 8
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else
        {
            let ret = u64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
            Ok((ret, 8))
        }
    }
}

impl Serializable for u32
{
    fn serialize(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 4
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else
        {
            let ret = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            Ok((ret, 4))
        }
    }
}

impl Serializable for u16
{
    fn serialize(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 2
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else
        {
            let ret = u16::from_be_bytes([data[0], data[1]]);
            Ok((ret, 2))
        }
    }
}

impl Serializable for u8
{
    fn serialize(&self) -> Vec<u8> {
        vec![*self]
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 1
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else
        {
            Ok((data[0], 1))
        }
    }
}

impl Serializable for f32
{
    fn serialize(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 4
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else
        {
            let ret = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            Ok((ret, 4))
        }
    }
}

impl Serializable for f64
{
    fn serialize(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 8
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else
        {
            let ret = f64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
            Ok((ret, 8))
        }
    }
}

impl Serializable for bool
{
    fn serialize(&self) -> Vec<u8> {
        match self {
            false => vec![0],
            true => vec![1]
        }
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 1
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else if data[0] == 0
        {
            Ok((false, 1))
        }
        else if data[0] == 1
        {
            Ok((true, 1))
        }
        else
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid bool value"))
        }
    }
}

impl Serializable for SystemTime
{
    fn serialize(&self) -> Vec<u8> {
        let duration = self.duration_since(SystemTime::UNIX_EPOCH).expect("System date earlier than UNIX_EPOCH whick is wrong because today is 2023");
        duration.as_secs().serialize()
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        let (secs, read) = u64::deserialize(data)?;
        let ret = SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(secs));
        match ret {
            Some(time) => Ok((time, read)),
            None => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid time")),
        }
    }
}

impl<const L: usize, T: Serializable> Serializable for [T;L]
{
    fn serialize(&self) -> Vec<u8> {
        return self.iter().flat_map(|x| x.serialize()).collect();
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        let mut ret: [T;L] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        let mut offset = 0;
        for i in 0..L
        {
            let (item, len) = T::deserialize(&data[offset..])?;
            ret[i] = item;
            offset += len;
        }
        Ok((ret, offset))
    }
}

impl<T: Serializable> Serializable for Option<T>
{
    fn serialize(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        match self {
            Some(item) => {
                ret.push(1);
                ret.extend(item.serialize());
            },
            None => {
                ret.push(0);
            }
        }
        ret
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        if data.len() < 1
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
        }
        else
        {
            match data[0] {
                0 => Ok((None, 1)),
                1 => {
                    let (item, len) = T::deserialize(&data[1..])?;
                    Ok((Some(item), len + 1))
                },
                _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid option type"))
            }
        }
    }
}