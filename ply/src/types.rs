/// Describes the recognized formats of a PLY file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatType {
    Ascii,
    BinaryBigEndian,
    BinaryLittleEndian,
}

/// Describes the recognized data types for property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float32,
    Float64,
}

/// Describes the recognized count data types for vector properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountType {
    Uint8,
    Uint16,
    Uint32,
}

/// Describes the PLY format and version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Format {
    pub format: FormatType,
    pub version: Vec<usize>,
}

/// Describes a PLY property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub name: String,
    pub count_data_type: Option<CountType>,
    pub data_type: DataType,
}

/// Describes a PLY element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub count: usize,
    pub properties: Vec<Property>,
}

/// Describes the PLY header..
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub format: Format,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyData {
    Int8(i8),
    Uint8(u8),
    Int16(i16),
    Uint16(u16),
    Int32(i32),
    Uint32(u32),
    Float32(f32),
    Float64(f64),
    Vint8(Vec<i8>),
    Vuint8(Vec<u8>),
    Vint16(Vec<i16>),
    Vuint16(Vec<u16>),
    Vint32(Vec<i32>),
    Vuint32(Vec<u32>),
    Vfloat32(Vec<f32>),
    Vfloat64(Vec<f64>),
}

impl From<i8> for PropertyData {
    fn from(value: i8) -> Self {
        PropertyData::Int8(value)
    }
}

impl From<u8> for PropertyData {
    fn from(value: u8) -> Self {
        PropertyData::Uint8(value)
    }
}

impl From<i16> for PropertyData {
    fn from(value: i16) -> Self {
        PropertyData::Int16(value)
    }
}

impl From<u16> for PropertyData {
    fn from(value: u16) -> Self {
        PropertyData::Uint16(value)
    }
}

impl From<i32> for PropertyData {
    fn from(value: i32) -> Self {
        PropertyData::Int32(value)
    }
}

impl From<u32> for PropertyData {
    fn from(value: u32) -> Self {
        PropertyData::Uint32(value)
    }
}

impl From<f32> for PropertyData {
    fn from(value: f32) -> Self {
        PropertyData::Float32(value)
    }
}

impl From<f64> for PropertyData {
    fn from(value: f64) -> Self {
        PropertyData::Float64(value)
    }
}

impl From<Vec<i8>> for PropertyData {
    fn from(value: Vec<i8>) -> Self {
        PropertyData::Vint8(value)
    }
}

impl From<Vec<u8>> for PropertyData {
    fn from(value: Vec<u8>) -> Self {
        PropertyData::Vuint8(value)
    }
}

impl From<Vec<i16>> for PropertyData {
    fn from(value: Vec<i16>) -> Self {
        PropertyData::Vint16(value)
    }
}

impl From<Vec<u16>> for PropertyData {
    fn from(value: Vec<u16>) -> Self {
        PropertyData::Vuint16(value)
    }
}

impl From<Vec<i32>> for PropertyData {
    fn from(value: Vec<i32>) -> Self {
        PropertyData::Vint32(value)
    }
}

impl From<Vec<u32>> for PropertyData {
    fn from(value: Vec<u32>) -> Self {
        PropertyData::Vuint32(value)
    }
}

impl From<Vec<f32>> for PropertyData {
    fn from(value: Vec<f32>) -> Self {
        PropertyData::Vfloat32(value)
    }
}

impl From<Vec<f64>> for PropertyData {
    fn from(value: Vec<f64>) -> Self {
        PropertyData::Vfloat64(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementData {
    pub properties: Vec<PropertyData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub elements: Vec<ElementData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ply {
    pub header: Header,
    pub body: Body,
}
