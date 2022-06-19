#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormatType {
    Ascii,
    BinaryLittleEndian,
    BinaryBigEndian,
}

impl Default for FormatType {
    fn default() -> Self {
        FormatType::Ascii
    }
}

impl std::fmt::Display for FormatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ascii => write!(f, "ascii"),
            Self::BinaryLittleEndian => write!(f, "binary_little_endian"),
            Self::BinaryBigEndian => write!(f, "binary_big_endian"),
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

impl std::fmt::Display for CountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8 => write!(f, "uint8"),
            Self::U16 => write!(f, "uint16"),
            Self::U32 => write!(f, "uint32"),
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

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I8 => write!(f, "int8"),
            Self::U8 => write!(f, "uint8"),
            Self::I16 => write!(f, "int16"),
            Self::U16 => write!(f, "uint16"),
            Self::I32 => write!(f, "int32"),
            Self::U32 => write!(f, "uint32"),
            Self::F32 => write!(f, "float32"),
            Self::F64 => write!(f, "float64"),
        }
    }
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone)]
pub struct ListPropertyDescriptor {
    pub name: String,
    pub count_type: CountType,
    pub data_type: DataType,
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub name: String,
    pub data_type: DataType,
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
