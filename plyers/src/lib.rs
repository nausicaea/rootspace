//! # Stanford PLY Parser
//!
//! ## Context-Free Grammar
//! ```
//! S -> H B  // S: Start symbol
//! H -> "ply" D "end_header"  // H: Header
//! B -> N
//! B -> N B  // B: Body
//! D -> F | E | P | G  // D: Declaration
//! D -> D D
//! F -> "format" A N  // F: Format declaration
//! E -> "element" B I  // E: Element declaration
//! P -> "property" C K | "property" "list" C C K  // P: Property declaration
//! G -> "comment" J  // G: Comment declaration
//! A -> "ascii" | "binary_little_endian" | "binary_big_endian"  // A: Format type
//! B -> "vertex" | "face" | "edge" | K  // B: Element type
//! C -> "char" | "uchar" | "short" | "ushort" | "int" | "uint" | "float" | "double"
//! N any integral or floating point number
//! I any integral number
//! K any identifier
//! J any string
//! ```


#[derive(Debug, Clone, Copy, PartialEq)]
enum FormatType {
    Ascii,
    BinaryLittleEndian,
    BinaryBigEndian,
}

impl Default for FormatType {
    fn default() -> Self {
        FormatType::Ascii
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ElementType {
    Vertex,
    Face,
    Edge,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PropertyType {
    Char,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Float,
    Double,
}

#[derive(Debug, Clone)]
struct ListPropertyDescriptor {
    pub name: String,
    pub count_type: PropertyType,
    pub property_type: PropertyType,
}

#[derive(Debug, Clone)]
struct PropertyDescriptor {
    pub name: String,
    pub property_type: PropertyType,
}

#[derive(Debug, Clone)]
struct ElementDescriptor {
    pub name: String,
    pub count: usize,
    pub properties: Vec<PropertyDescriptor>,
    pub list_properties: Vec<ListPropertyDescriptor>,
}

#[derive(Debug, Clone)]
struct PlyDescriptor {
    pub format_type: FormatType,
    pub format_version: String,
    pub elements: Vec<ElementDescriptor>,
}

impl Default for PlyDescriptor {
    fn default() -> Self {
        PlyDescriptor {
            format_type: FormatType::Ascii,
            format_version: String::from("1.0"),
            elements: Vec::default(),
        }
    }
}

#[derive(Debug)]
struct Ply {
    pub descriptor: PlyDescriptor,
    pub comments: Vec<String>,
    pub property_data: Vec<u8>,
    pub property_list_data: Vec<u8>,
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
    fn element_type_provides_these_variants() {
        let _ = ElementType::Vertex;
        let _ = ElementType::Face;
        let _ = ElementType::Edge;
        let _ = ElementType::Custom(String::from("identifier"));
    }

    #[test]
    fn property_type_provides_these_variants() {
        let _ = PropertyType::Char;
        let _ = PropertyType::UChar;
        let _ = PropertyType::Short;
        let _ = PropertyType::UShort;
        let _ = PropertyType::Int;
        let _ = PropertyType::UInt;
        let _ = PropertyType::Float;
        let _ = PropertyType::Double;
        
    }

    #[test]
    fn ply_data_container_has_the_following_structure() {
        let _ = Ply {
            descriptor: PlyDescriptor::default(),
            comments: Vec::<String>::new(),
            property_data: Vec::<u8>::new(),
            property_list_data: Vec::<u8>::new(),
        };
    }

    #[test]
    fn ply_descriptor_has_the_following_structure() {
        let _ = PlyDescriptor {
            format_type: FormatType::Ascii,
            format_version: String::from("1.0"),
            elements: Vec::<ElementDescriptor>::new(),
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
            property_type: PropertyType::Float,
        };
    }

    #[test]
    fn list_property_descriptor_has_the_following_structure() {
        let _ = ListPropertyDescriptor {
            name: String::from("x"),
            count_type: PropertyType::UShort,
            property_type: PropertyType::Float,
        };
    }
}
