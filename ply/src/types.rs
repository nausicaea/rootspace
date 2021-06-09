//! The module `types` defines internal representations of parts of a Stanford PLY file.

use std::collections::HashSet;

use crate::impl_property_data;

/// Describes the recognized formats of a PLY file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatType {
    /// Specifies that the ply file in question uses an ascii encoding.
    Ascii,
    /// Specifies that the ply file in question uses a big-endian binary encoding.
    BinaryBigEndian,
    /// Specifies that the ply file in question uses a little-endian binary encoding.
    BinaryLittleEndian,
}

/// Describes the recognized data types for property values.
#[allow(missing_docs)]
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
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountType {
    Uint8,
    Uint16,
    Uint32,
}

/// Describes the PLY format and version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Format {
    /// Holds the file encoding format.
    pub format: FormatType,
    /// Holds the ply file format version. Only 1.0 is known and supported.
    pub version: Vec<usize>,
}

/// Describes a PLY property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    /// Holds the name of the property. Must be unique per `Element` to be of any use.
    pub name: String,
    /// A property may be either scalar or vector. In the latter case, a count preceeds the actual
    /// property data. This field specify the data type of that count value.
    pub count_data_type: Option<CountType>,
    /// Specifies the data type of the property value(s).
    pub data_type: DataType,
}

/// Describes a PLY element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// Holds the name of the element. Must be unique per ply file to be of any use.
    pub name: String,
    /// Holds the number of occurrences of this element within the body of the ply file.
    pub count: usize,
    /// Holds the properties of this element.
    pub properties: Vec<Property>,
}

impl Element {
    /// Returns `true` if a property name occurs more than once.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate ply;
    ///
    /// use ply::types::{Element, Property, DataType};
    ///
    /// let e = Element {
    ///     name: "vertex".into(),
    ///     count: 32,
    ///     properties: vec![
    ///         Property {
    ///             name: "x".into(),
    ///             count_data_type: None,
    ///             data_type: DataType::Float32,
    ///         },
    ///         Property {
    ///             name: "x".into(),
    ///             count_data_type: None,
    ///             data_type: DataType::Float32,
    ///         },
    ///     ],
    /// };
    ///
    /// assert!(e.has_duplicate_properties());
    /// ```
    pub fn has_duplicate_properties(&self) -> bool {
        let mut unique = HashSet::new();
        !self.properties.iter().all(|p| unique.insert(p.name.clone()))
    }

    /// Returns the last scalar property that matches any of the specified names. Also returns the
    /// index of the property which can be used to obtain the corresponding data from the body of
    /// the ply file.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate ply;
    ///
    /// use ply::types::{Element, Property, CountType, DataType};
    ///
    /// let a = Property {
    ///     name: "a".into(),
    ///     count_data_type: None,
    ///     data_type: DataType::Float32,
    /// };
    ///
    /// let b = Property {
    ///     name: "b".into(),
    ///     count_data_type: Some(CountType::Uint8),
    ///     data_type: DataType::Float32,
    /// };
    ///
    /// let e = Element {
    ///     name: "vertex".into(),
    ///     count: 32,
    ///     properties: vec![a.clone(), b.clone()],
    /// };
    ///
    /// assert_eq!(e.scalar_property(&["a", "prop_a"]), Some((0usize, &a)));
    /// assert_eq!(e.scalar_property(&["b", "prop_b"]), None);
    /// ```
    pub fn scalar_property(&self, names: &[&str]) -> Option<(usize, &Property)> {
        self.properties
            .iter()
            .enumerate()
            .filter(|(_, p)| p.count_data_type.is_none() && names.iter().any(|n| p.name == *n))
            .last()
    }

    /// Returns the last vector property that matches any of the specified names. Also returns the
    /// index of the property which can be used to obtain the corresponding data from the body of
    /// the ply file.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate ply;
    ///
    /// use ply::types::{Element, Property, CountType, DataType};
    ///
    /// let a = Property {
    ///     name: "a".into(),
    ///     count_data_type: None,
    ///     data_type: DataType::Float32,
    /// };
    ///
    /// let b = Property {
    ///     name: "b".into(),
    ///     count_data_type: Some(CountType::Uint8),
    ///     data_type: DataType::Float32,
    /// };
    ///
    /// let e = Element {
    ///     name: "vertex".into(),
    ///     count: 32,
    ///     properties: vec![a.clone(), b.clone()],
    /// };
    ///
    /// assert_eq!(e.vector_property(&["a", "prop_a"]), None);
    /// assert_eq!(e.vector_property(&["b", "prop_b"]), Some((1usize, &b)));
    /// ```
    pub fn vector_property(&self, names: &[&str]) -> Option<(usize, &Property)> {
        self.properties
            .iter()
            .enumerate()
            .filter(|(_, p)| p.count_data_type.is_some() && names.iter().any(|n| p.name == *n))
            .last()
    }
}

/// Describes the PLY header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Denotes the file format.
    pub format: Format,
    /// Specifies all elements (and their properties) that will appear in this file.
    pub elements: Vec<Element>,
}

impl Header {
    /// Returns `true` if an element name occurs more than once.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate ply;
    ///
    /// use ply::types::{Header, Format, FormatType, Element};
    ///
    /// let h = Header {
    ///     format: Format {
    ///         format: FormatType::Ascii,
    ///         version: vec![1, 0],
    ///     },
    ///     elements: vec![
    ///         Element {
    ///             name: "vertex".into(),
    ///             count: 32,
    ///             properties: Vec::new(),
    ///         },
    ///         Element {
    ///             name: "vertex".into(),
    ///             count: 10,
    ///             properties: Vec::new(),
    ///         },
    ///     ],
    /// };
    ///
    /// assert!(h.has_duplicate_elements());
    /// ```
    pub fn has_duplicate_elements(&self) -> bool {
        let mut unique = HashSet::new();
        !self.elements.iter().all(|e| unique.insert(e.name.clone()))
    }

    /// Returns the last element that matches any of the specified names. Also returns the index of
    /// the element which can be used to obtain the corresponding data from the body of the ply
    /// file.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate ply;
    ///
    /// use ply::types::{Header, Format, FormatType, Element};
    ///
    /// let a = Element {
    ///     name: "vertex".into(),
    ///     count: 32,
    ///     properties: Vec::new(),
    /// };
    ///
    /// let b = Element {
    ///     name: "face".into(),
    ///     count: 100,
    ///     properties: Vec::new(),
    /// };
    ///
    /// let h = Header {
    ///     format: Format {
    ///         format: FormatType::Ascii,
    ///         version: vec![1, 0],
    ///     },
    ///     elements: vec![a.clone(), b.clone()],
    /// };
    ///
    /// assert_eq!(h.element(&["vertex", "vertices"]), Some((0usize, &a)));
    /// ```
    pub fn element(&self, names: &[&str]) -> Option<(usize, &Element)> {
        self.elements
            .iter()
            .enumerate()
            .filter(|(_, e)| names.iter().any(|n| e.name == *n))
            .last()
    }
}

impl_property_data! {
    /// Holds data of a single property.
    #[allow(missing_docs)]
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
}

macro_rules! impl_conversions {
    ($type:ty) => {
        impl std::convert::TryFrom<$crate::types::PropertyData> for $type {
            type Error = ();

            fn try_from(value: $crate::types::PropertyData) -> Result<Self, Self::Error> {
                use $crate::types::PropertyData::*;
                match value {
                    Int8(t) => Ok(t as $type),
                    Uint8(t) => Ok(t as $type),
                    Int16(t) => Ok(t as $type),
                    Uint16(t) => Ok(t as $type),
                    Int32(t) => Ok(t as $type),
                    Uint32(t) => Ok(t as $type),
                    Float32(t) => Ok(t as $type),
                    Float64(t) => Ok(t as $type),
                    _ => Err(()),
                }
            }
        }

        impl std::convert::TryFrom<&$crate::types::PropertyData> for $type {
            type Error = ();

            fn try_from(value: &$crate::types::PropertyData) -> Result<Self, Self::Error> {
                use $crate::types::PropertyData::*;
                match value {
                    Int8(ref t) => Ok(*t as $type),
                    Uint8(ref t) => Ok(*t as $type),
                    Int16(ref t) => Ok(*t as $type),
                    Uint16(ref t) => Ok(*t as $type),
                    Int32(ref t) => Ok(*t as $type),
                    Uint32(ref t) => Ok(*t as $type),
                    Float32(ref t) => Ok(*t as $type),
                    Float64(ref t) => Ok(*t as $type),
                    _ => Err(()),
                }
            }
        }

        impl std::convert::TryFrom<$crate::types::PropertyData> for Vec<$type> {
            type Error = ();

            fn try_from(value: $crate::types::PropertyData) -> Result<Self, Self::Error> {
                use $crate::types::PropertyData::*;
                match value {
                    Vint8(t) => Ok(t.into_iter().map(|el| el as $type).collect()),
                    Vuint8(t) => Ok(t.into_iter().map(|el| el as $type).collect()),
                    Vint16(t) => Ok(t.into_iter().map(|el| el as $type).collect()),
                    Vuint16(t) => Ok(t.into_iter().map(|el| el as $type).collect()),
                    Vint32(t) => Ok(t.into_iter().map(|el| el as $type).collect()),
                    Vuint32(t) => Ok(t.into_iter().map(|el| el as $type).collect()),
                    Vfloat32(t) => Ok(t.into_iter().map(|el| el as $type).collect()),
                    Vfloat64(t) => Ok(t.into_iter().map(|el| el as $type).collect()),
                    _ => Err(()),
                }
            }
        }

        impl std::convert::TryFrom<&$crate::types::PropertyData> for Vec<$type> {
            type Error = ();

            fn try_from(value: &$crate::types::PropertyData) -> Result<Self, Self::Error> {
                use $crate::types::PropertyData::*;
                match value {
                    Vint8(ref t) => Ok(t.iter().map(|el| *el as $type).collect()),
                    Vuint8(ref t) => Ok(t.iter().map(|el| *el as $type).collect()),
                    Vint16(ref t) => Ok(t.iter().map(|el| *el as $type).collect()),
                    Vuint16(ref t) => Ok(t.iter().map(|el| *el as $type).collect()),
                    Vint32(ref t) => Ok(t.iter().map(|el| *el as $type).collect()),
                    Vuint32(ref t) => Ok(t.iter().map(|el| *el as $type).collect()),
                    Vfloat32(ref t) => Ok(t.iter().map(|el| *el as $type).collect()),
                    Vfloat64(ref t) => Ok(t.iter().map(|el| *el as $type).collect()),
                    _ => Err(()),
                }
            }
        }
    };
}

impl_conversions!(i8);
impl_conversions!(u8);
impl_conversions!(i16);
impl_conversions!(u16);
impl_conversions!(i32);
impl_conversions!(u32);
impl_conversions!(f32);
impl_conversions!(f64);

/// Holds data of a single element.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementData {
    /// Holds all property data for that element.
    pub properties: Vec<PropertyData>,
}

/// Holds data of all occurrences of each element specified in the header, i.e. all data.
#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    /// Holds all element (and property) data for that particular ply file.
    pub elements: Vec<Vec<ElementData>>,
}

impl Body {
    /// Given an element index and a mapper closure, calls the mapper for each instance of the
    /// supplied element. This allows to map the ply data to other representations.
    pub fn generate<T, F>(&self, element: usize, mapper: F) -> Vec<T>
    where
        F: Fn(&[PropertyData]) -> T,
    {
        self.elements[element].iter().map(|i| mapper(&i.properties)).collect()
    }
}

/// Describes an in-memory representation of the PLY file format.
#[derive(Debug, Clone, PartialEq)]
pub struct Ply {
    /// Specifies the ply header.
    pub header: Header,
    /// Specifies the ply data body. Cannot be parsed without the header.
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

    #[test]
    fn get_element() {
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

        let r = h.element(&["vertex"]);
        assert_eq!(
            r,
            Some((
                0,
                &Element {
                    name: "vertex".into(),
                    count: 0,
                    properties: Vec::new()
                }
            ))
        );
    }

    #[test]
    fn get_scalar_property() {
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

        let r = e.scalar_property(&["y"]);
        assert_eq!(
            r,
            Some((
                1,
                &Property {
                    name: "y".into(),
                    count_data_type: None,
                    data_type: DataType::Float32
                }
            ))
        );
    }

    #[test]
    fn get_vector_property() {
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
                    name: "vertex_index".into(),
                    count_data_type: Some(CountType::Uint8),
                    data_type: DataType::Float32,
                },
            ],
        };

        let r = e.vector_property(&["vertex_index"]);
        assert_eq!(
            r,
            Some((
                1,
                &Property {
                    name: "vertex_index".into(),
                    count_data_type: Some(CountType::Uint8),
                    data_type: DataType::Float32
                }
            ))
        );
    }
}
