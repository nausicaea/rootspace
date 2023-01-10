use std::collections::{BTreeMap, HashSet};

pub const VERTEX_ELEMENT: &'static str = "vertex";
pub const FACE_ELEMENT: &'static str = "face";

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormatType {
    Ascii,
    BinaryLittleEndian,
    BinaryBigEndian,
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CountType {
    U8,
    U16,
    U32,
    U64,
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
    I64,
    U64,
    F32,
    F64,
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary, PartialEq))]
#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub data_type: DataType,
    pub name: String,
    pub comments: Vec<CommentDescriptor>,
    pub obj_info: Vec<ObjInfoDescriptor>,
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary, PartialEq))]
#[derive(Debug, Clone)]
pub struct ListPropertyDescriptor {
    pub count_type: CountType,
    pub data_type: DataType,
    pub name: String,
    pub comments: Vec<CommentDescriptor>,
    pub obj_info: Vec<ObjInfoDescriptor>,
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary, PartialEq))]
#[derive(Debug, Clone)]
pub struct ElementDescriptor {
    pub name: String,
    pub count: usize,
    pub properties: Vec<PropertyDescriptor>,
    pub list_properties: Vec<ListPropertyDescriptor>,
    pub comments: Vec<CommentDescriptor>,
    pub obj_info: Vec<ObjInfoDescriptor>,
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary, PartialEq))]
#[derive(Debug, Clone)]
pub struct CommentDescriptor(pub String);

#[cfg_attr(test, derive(proptest_derive::Arbitrary, PartialEq))]
#[derive(Debug, Clone)]
pub struct ObjInfoDescriptor(pub String);

#[cfg_attr(test, derive(proptest_derive::Arbitrary, PartialEq))]
#[derive(Debug, Clone)]
pub struct PlyDescriptor {
    pub format_type: FormatType,
    pub elements: Vec<ElementDescriptor>,
    pub comments: Vec<CommentDescriptor>,
    pub obj_info: Vec<ObjInfoDescriptor>,
}

impl Default for PlyDescriptor {
    fn default() -> Self {
        PlyDescriptor {
            format_type: FormatType::Ascii,
            elements: Vec::default(),
            comments: Vec::default(),
            obj_info: Vec::default(),
        }
    }
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    Triangles,
    Quads,
    Mixed,
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary, PartialEq))]
#[derive(Debug)]
pub struct Ply<T> {
    pub descriptor: PlyDescriptor,
    pub data: BTreeMap<String, (Vec<T>, Vec<Vec<T>>)>,
}

impl<T> Ply<T> {
    pub fn face_type(&self) -> Option<Primitive> {
        if !self.data.contains_key(FACE_ELEMENT) {
            return None;
        }

        if self.data[FACE_ELEMENT].1.is_empty() {
            return None;
        }

        let primitives: HashSet<usize> = self.data[FACE_ELEMENT].1.iter().map(|f| f.len()).collect();

        if primitives.iter().all(|&p| p == 3) {
            Some(Primitive::Triangles)
        } else if primitives.iter().all(|&p| p == 4) {
            Some(Primitive::Quads)
        } else {
            Some(Primitive::Mixed)
        }
    }
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
            data: BTreeMap::<String, (Vec<f32>, Vec<Vec<f32>>)>::default(),
        };
    }

    #[test]
    fn ply_descriptor_has_the_following_structure() {
        let _ = PlyDescriptor {
            format_type: FormatType::Ascii,
            elements: Vec::<ElementDescriptor>::new(),
            comments: Vec::<CommentDescriptor>::new(),
            obj_info: Vec::<ObjInfoDescriptor>::new(),
        };
    }

    #[test]
    fn element_descriptor_has_the_following_structure() {
        let _ = ElementDescriptor {
            name: String::from("vertex"),
            count: 0usize,
            properties: Vec::<PropertyDescriptor>::new(),
            list_properties: Vec::<ListPropertyDescriptor>::new(),
            comments: Vec::<CommentDescriptor>::new(),
            obj_info: Vec::<ObjInfoDescriptor>::new(),
        };
    }

    #[test]
    fn property_descriptor_has_the_following_structure() {
        let _ = PropertyDescriptor {
            name: String::from("x"),
            data_type: DataType::F32,
            comments: Vec::<CommentDescriptor>::new(),
            obj_info: Vec::<ObjInfoDescriptor>::new(),
        };
    }

    #[test]
    fn list_property_descriptor_has_the_following_structure() {
        let _ = ListPropertyDescriptor {
            name: String::from("i"),
            count_type: CountType::U16,
            data_type: DataType::F32,
            comments: Vec::<CommentDescriptor>::new(),
            obj_info: Vec::<ObjInfoDescriptor>::new(),
        };
    }
}
