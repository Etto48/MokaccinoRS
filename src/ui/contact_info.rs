
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContactInfo
{
    name: String,
}


impl ContactInfo
{
    pub fn new(name: &str) -> Self
    {
        Self { name: name.to_string() }
    }

    pub fn name(&self) -> &str
    {
        &self.name
    }
}