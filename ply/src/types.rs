use std::collections::HashSet;

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

impl Element {
    pub fn has_duplicate_properties(&self) -> bool {
        let mut unique = HashSet::new();
        !self.properties.iter().all(|p| unique.insert(p.name.clone()))
    }
}

/// Describes the PLY header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub format: Format,
    pub elements: Vec<Element>,
}

impl Header {
    pub fn has_duplicate_elements(&self) -> bool {
        let mut unique = HashSet::new();
        !self.elements.iter().all(|e| unique.insert(e.name.clone()))
    }
}

/// Holds data of a single property.
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

/// Holds data of a single element.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementData {
    pub properties: Vec<PropertyData>,
}

/// Holds data of all occurrences of each element specified in the header, i.e. all data.
#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub elements: Vec<Vec<ElementData>>,
}

/// Describes an in-memory representation of the PLY file format.
#[derive(Debug, Clone, PartialEq)]
pub struct Ply {
    pub header: Header,
    pub body: Body,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_properties() {
        let e = Element {
            name: "vertex".into(),
            count: 1,
            properties: vec![
                Property {
                    name: "x".into(),
                    count_data_type: None,
                    data_type: DataType::Float32,
                },
                Property {
                    name: "x".into(),
                    count_data_type: None,
                    data_type: DataType::Float32,
                },
            ],
        };

        assert!(e.has_duplicate_properties());
    }

    #[test]
    fn nonduplicate_properties() {
        let e = Element {
            name: "vertex".into(),
            count: 1,
            properties: vec![
                Property {
                    name: "x".into(),
                    count_data_type: None,
                    data_type: DataType::Float32,
                },
                Property {
                    name: "y".into(),
                    count_data_type: None,
                    data_type: DataType::Float32,
                },
            ],
        };

        assert!(!e.has_duplicate_properties());
    }

    #[test]
    fn duplicate_elements() {
        let h = Header {
            format: Format {
                format: FormatType::Ascii,
                version: vec![1, 0],
            },
            elements: vec![
                Element {
                    name: "vertex".into(),
                    count: 0,
                    properties: Vec::new(),
                },
                Element {
                    name: "vertex".into(),
                    count: 0,
                    properties: Vec::new(),
                },
            ],
        };

        assert!(h.has_duplicate_elements());
    }

    #[test]
    fn nonduplicate_elements() {
        let h = Header {
            format: Format {
                format: FormatType::Ascii,
                version: vec![1, 0],
            },
            elements: vec![
                Element {
                    name: "vertex".into(),
                    count: 0,
                    properties: Vec::new(),
                },
                Element {
                    name: "face".into(),
                    count: 0,
                    properties: Vec::new(),
                },
            ],
        };

        assert!(!h.has_duplicate_elements());
    }
}
