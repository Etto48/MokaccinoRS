pub mod serializable;

pub use crate::serializable::Serializable;
pub use serializable_derive::Serializable;

#[cfg(test)]
mod tests
{
    use super::Serializable;
    
    #[derive(Serializable, Debug, PartialEq)]
    pub struct NamedTestStruct
    {
        a: u32,
        b: u16,
        c: String
    }
    
    #[test]
    fn serialize_and_deserialize_named()
    {
        let test_struct = NamedTestStruct { a: 0x12345678, b: 0x9ABC, c: "Hello world".to_string() };
        let serialized = test_struct.serialize();
        let (deserialized, bytes_read) = NamedTestStruct::deserialize(&serialized).unwrap();
        assert_eq!(test_struct, deserialized);
        assert_eq!(serialized.len(), bytes_read);
    }

    #[derive(Serializable, Debug, PartialEq)]
    pub struct UnnamedTestStruct(u32, u16, String);
    #[test]
    fn serialize_and_deserialize_unnamed()
    {
        let test_struct = UnnamedTestStruct ( 0x12345678, 0x9ABC, "Hello world".to_string() );
        let serialized = test_struct.serialize();
        let (deserialized, bytes_read) = UnnamedTestStruct::deserialize(&serialized).unwrap();
        assert_eq!(test_struct, deserialized);
        assert_eq!(serialized.len(), bytes_read);
    }

    #[derive(Serializable, Debug, PartialEq)]
    pub struct UnitTestStruct;
    #[test]
    fn serialize_and_deserialize_unit()
    {
        let test_struct = UnitTestStruct;
        let serialized = test_struct.serialize();
        let (deserialized, bytes_read) = UnitTestStruct::deserialize(&serialized).unwrap();
        assert_eq!(test_struct, deserialized);
        assert_eq!(serialized.len(), bytes_read);
    }

    #[derive(Serializable, Debug, PartialEq)]
    pub enum TestEnum
{
    A(u32),
    B(u16),
    C(String),
    D,
    E{f: u32, g: u16, h: String},
}
    #[test]
    fn serialize_and_deserialize_enum()
    {
        let test_enum = TestEnum::C("Hello world".to_string());
        let serialized = test_enum.serialize();
        let (deserialized, bytes_read) = TestEnum::deserialize(&serialized).unwrap();
        assert_eq!(test_enum, deserialized);
        assert_eq!(serialized.len(), bytes_read);
    }

    #[derive(Serializable, Debug, PartialEq)]
    pub struct TestStructWithVec
    {
        a: u32,
        b: u16,
        c: Vec<u8>
    }
    #[test]
    fn serialize_and_deserialize_vec()
    {
        let test_struct = TestStructWithVec { a: 0x12345678, b: 0x9ABC, c: vec![1,2,3,4,5,6,7,8,9] };
        let serialized = test_struct.serialize();
        let (deserialized, bytes_read) = TestStructWithVec::deserialize(&serialized).unwrap();
        assert_eq!(test_struct, deserialized);
        assert_eq!(serialized.len(), bytes_read);
    }
}