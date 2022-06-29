pub const KEYWORDS: &'static [&'static [u8]] = &[b"format", b"element", b"property list", b"property", b"comment", b"obj_info"];
pub const FORMAT_TYPES: &'static [&'static [u8]] = &[b"ascii", b"binary_little_endian", b"binary_big_endian"];
pub const COUNT_TYPES: &'static [&'static [u8]] = &[b"uint8", b"uint16", b"uint32", b"uchar", b"ushort", b"uint"];
pub const DATA_TYPES: &'static [&'static [u8]] = &[
    b"uint8", b"uchar",
    b"int8", b"char",
    b"uint16", b"ushort",
    b"int16", b"short",
    b"uint32", b"uint",
    b"int32", b"int",
    b"float32", b"float",
    b"float64", b"double",
];

#[derive(Debug, Clone, thiserror::Error)]
#[error("expected one byte sequence of {:?}, but received {:?}", .expected, .received)]
pub struct FromBytesError {
    received: Vec<u8>,
    expected: &'static [&'static [u8]],
}

impl FromBytesError {
    fn new(received: Vec<u8>, expected: &'static [&'static [u8]]) -> Self {
        FromBytesError { received, expected }
    }
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Format,
    Element,
    Property,
    ListProperty,
    Comment,
    ObjInfo,
}

impl Keyword {
    pub fn to_bytes(&self) -> &'static [u8] {
        Into::<&'static [u8]>::into(*self)
    }

    pub fn try_from_bytes(s: &[u8]) -> Result<Self, FromBytesError> {
        TryFrom::<&[u8]>::try_from(s)
    }
}

impl Into<&'static [u8]> for Keyword {
    fn into(self) -> &'static [u8] {
        match self {
            Keyword::Format => b"format",
            Keyword::Element => b"element",
            Keyword::Property => b"property",
            Keyword::ListProperty => b"property list",
            Keyword::Comment => b"comment",
            Keyword::ObjInfo => b"obj_info",
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for Keyword {
    type Error = FromBytesError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value {
            b"format" => Ok(Keyword::Format),
            b"element" => Ok(Keyword::Element),
            b"property" => Ok(Keyword::Property),
            b"property list" => Ok(Keyword::ListProperty),
            b"comment" => Ok(Keyword::Comment),
            b"obj_info" => Ok(Keyword::ObjInfo),
            b => Err(FromBytesError::new(b.iter().copied().collect(), KEYWORDS)),
        }
    }
}
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormatType {
    Ascii,
    BinaryLittleEndian,
    BinaryBigEndian,
}

impl FormatType {
    pub fn to_bytes(&self) -> &'static [u8] {
        Into::<&'static [u8]>::into(*self)
    }

    pub fn try_from_bytes(s: &[u8]) -> Result<Self, FromBytesError> {
        TryFrom::<&[u8]>::try_from(s)
    }
}

impl Default for FormatType {
    fn default() -> Self {
        FormatType::Ascii
    }
}

impl Into<&'static [u8]> for FormatType {
    fn into(self) -> &'static [u8] {
        match self {
            Self::Ascii => b"ascii",
            Self::BinaryLittleEndian => b"binary_little_endian",
            Self::BinaryBigEndian => b"binary_big_endian",
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for FormatType {
    type Error = FromBytesError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value {
            b"ascii" => Ok(FormatType::Ascii),
            b"binary_little_endian" => Ok(FormatType::BinaryLittleEndian),
            b"binary_big_endian" => Ok(FormatType::BinaryBigEndian),
            b => Err(FromBytesError::new(b.iter().copied().collect(), FORMAT_TYPES)),
        }
    }
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CountType {
    U8,
    U16,
    U32,
}

impl CountType {
    pub fn to_bytes(&self) -> &'static [u8] {
        Into::<&'static [u8]>::into(*self)
    }

    pub fn try_from_bytes(s: &[u8]) -> Result<Self, FromBytesError> {
        TryFrom::<&[u8]>::try_from(s)
    }
}

impl Into<&'static [u8]> for CountType {
    fn into(self) -> &'static [u8] {
        match self {
            CountType::U8 => b"uint8",
            CountType::U16 => b"uint16",
            CountType::U32 => b"uint32",
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for CountType {
    type Error = FromBytesError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value {
            b"uint8" | b"uchar" => Ok(CountType::U8),
            b"uint16" | b"ushort" => Ok(CountType::U16),
            b"uint32" | b"uint" => Ok(CountType::U32),
            b => Err(FromBytesError::new(b.iter().copied().collect(), COUNT_TYPES)),
        }
    }
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    F32,
    F64,
}

impl DataType {
    pub fn to_bytes(&self) -> &'static [u8] {
        Into::<&'static [u8]>::into(*self)
    }

    pub fn try_from_bytes(s: &[u8]) -> Result<Self, FromBytesError> {
        TryFrom::<&[u8]>::try_from(s)
    }
}

impl Into<&'static [u8]> for DataType {
    fn into(self) -> &'static [u8] {
        match self {
            DataType::U8 => b"uint8",
            DataType::I8 => b"int8",
            DataType::U16 => b"uint16",
            DataType::I16 => b"int16",
            DataType::U32 => b"uint32",
            DataType::I32 => b"int32",
            DataType::F32 => b"float32",
            DataType::F64 => b"float64",
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for DataType {
    type Error = FromBytesError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match value {
            b"uint8" | b"uchar" => Ok(DataType::U8),
            b"int8" | b"char" => Ok(DataType::I8),
            b"uint16" | b"ushort" => Ok(DataType::U16),
            b"int16" | b"short" => Ok(DataType::I16),
            b"uint32" | b"uint" => Ok(DataType::U32),
            b"int32" | b"int" => Ok(DataType::I32),
            b"float32" | b"float" => Ok(DataType::F32),
            b"float64" | b"double" => Ok(DataType::F64),
            b => Err(FromBytesError::new(b.iter().copied().collect(), DATA_TYPES)),
        }
    }
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub data_type: DataType,
    pub name: String,
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone)]
pub struct ListPropertyDescriptor {
    pub count_type: CountType,
    pub data_type: DataType,
    pub name: String,
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone)]
pub struct ElementDescriptor {
    pub name: String,
    pub count: usize,
    pub properties: Vec<PropertyDescriptor>,
    pub list_properties: Vec<ListPropertyDescriptor>,
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone)]
pub struct CommentDescriptor(pub String);

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone)]
pub struct ObjInfoDescriptor(pub String);

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone)]
pub struct PlyDescriptor {
    pub format_type: FormatType,
    #[cfg_attr(test, proptest(regex = r"1\.0"))]
    pub format_version: String,
    pub elements: Vec<ElementDescriptor>,
    pub comments: Vec<String>,
    pub obj_info: Vec<String>,
}

impl Default for PlyDescriptor {
    fn default() -> Self {
        PlyDescriptor {
            format_type: FormatType::Ascii,
            format_version: String::from("1.0"),
            elements: Vec::default(),
            comments: Vec::default(),
            obj_info: Vec::default(),
        }
    }
}

#[derive(Debug)]
pub struct Ply {
    pub descriptor: PlyDescriptor,
    pub property_data: Vec<u8>,
    pub list_property_data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_type_provides_these_variants() {
        let _ = FormatType::Ascii;
        let _ = FormatType::BinaryLittleEndian;
        let _ = FormatType::BinaryBigEndian;
    }

    #[test]
    fn data_type_provides_these_variants() {
        let _ = DataType::I8;
        let _ = DataType::U8;
        let _ = DataType::I16;
        let _ = DataType::U16;
        let _ = DataType::I32;
        let _ = DataType::U32;
        let _ = DataType::F32;
        let _ = DataType::F64;
    }

    #[test]
    fn count_type_provides_these_variants() {
        let _ = CountType::U8;
        let _ = CountType::U16;
        let _ = CountType::U32;
    }

    #[test]
    fn ply_data_container_has_the_following_structure() {
        let _ = Ply {
            descriptor: PlyDescriptor::default(),
            property_data: Vec::<u8>::new(),
            list_property_data: Vec::<u8>::new(),
        };
    }

    #[test]
    fn ply_descriptor_has_the_following_structure() {
        let _ = PlyDescriptor {
            format_type: FormatType::Ascii,
            format_version: String::from("1.0"),
            elements: Vec::<ElementDescriptor>::new(),
            comments: Vec::<String>::new(),
            obj_info: Vec::<String>::new(),
        };
    }

    #[test]
    fn element_descriptor_has_the_following_structure() {
        let _ = ElementDescriptor {
            name: String::from("vertex"),
            count: 0usize,
            properties: Vec::<PropertyDescriptor>::new(),
            list_properties: Vec::<ListPropertyDescriptor>::new(),
        };
    }

    #[test]
    fn property_descriptor_has_the_following_structure() {
        let _ = PropertyDescriptor {
            name: String::from("x"),
            data_type: DataType::F32,
        };
    }

    #[test]
    fn list_property_descriptor_has_the_following_structure() {
        let _ = ListPropertyDescriptor {
            name: String::from("i"),
            count_type: CountType::U16,
            data_type: DataType::F32,
        };
    }
}
