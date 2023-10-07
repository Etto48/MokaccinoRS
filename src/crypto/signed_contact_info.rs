use serializable::Serializable;

use crate::network::ContactInfo;

use super::{PrivateKey, PublicKey};

#[derive(Serializable, Clone, Debug, PartialEq)]
pub struct SignedContactInfo
{
    info: Vec<u8>,
    signature: Vec<u8>,
}

impl SignedContactInfo
{
    pub fn from_contact_info(contact_info: ContactInfo, private_key: &PrivateKey) -> Self
    {
        let info = contact_info.serialize();
        let signature = private_key.sign(&info);
        Self
        {
            info,
            signature,
        }
    }

    pub fn into_contact_info(self, public_key: &PublicKey) -> std::io::Result<ContactInfo>
    {
        if public_key.verify(&self.info, &self.signature)
        {
            let (contact_info, len) = ContactInfo::deserialize(&self.info)?;
            if len == self.info.len()
            {
                Ok(contact_info)
            }
            else
            {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data length"))
            }
        }
        else
        {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid signature"))
        }
    }
}