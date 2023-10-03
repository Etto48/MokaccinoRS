use serializable::Serializable;

use crate::network::ContactInfo;

use super::{PrivateKey, PublicKey};

pub struct SignedContactInfo
{
    contact_info_data: Vec<u8>,
    signature: Vec<u8>,
}

impl SignedContactInfo
{
    pub fn from_contact_info(contact_info: ContactInfo, private_key: &PrivateKey) -> Self
    {
        let contact_info_data = contact_info.serialize();
        let signature = private_key.sign(&contact_info_data);
        Self { contact_info_data, signature }
    }

    pub fn into_contact_info(self, public_key: &PublicKey) -> std::io::Result<ContactInfo>
    {
        if public_key.verify(&self.contact_info_data, &self.signature)
        {
            ContactInfo::deserialize(&self.contact_info_data).map(|(contact_info, _)| contact_info)
        }
        else
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid signature"))
        }
    }
}

impl Serializable for SignedContactInfo
{
    fn serialize(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.extend(self.contact_info_data.serialize());
        ret.extend(self.signature.serialize());
        ret
    }

    fn deserialize(data: &[u8]) -> std::io::Result<(Self,usize)> {
        let (contact_info_data, bytes_read) = Vec::<u8>::deserialize(data)?;
        let (signature, bytes_read) = Vec::<u8>::deserialize(&data[bytes_read..])?;
        Ok((Self { contact_info_data, signature }, bytes_read))
    }
}