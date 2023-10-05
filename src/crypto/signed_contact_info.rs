use serializable::Serializable;

use crate::network::ContactInfo;

use super::{PrivateKey, PublicKey};

#[derive(Serializable, Clone, Debug, PartialEq)]
pub struct SignedContactInfo
{
}

impl SignedContactInfo
{
    pub fn from_contact_info(contact_info: ContactInfo, private_key: &PrivateKey) -> Self
    {
        todo!()
    }

    pub fn into_contact_info(self, public_key: &PublicKey) -> std::io::Result<ContactInfo>
    {
        todo!()
    }
}