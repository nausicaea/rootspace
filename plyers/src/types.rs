use std::collections::{BTreeMap, HashSet};

use either::Either;

pub const VERTEX_ELEMENT: &'static str = "vertex";
pub const FACE_ELEMENT: &'static str = "face";
pub const X_PROPERTY: &'static str = "x";
pub const Y_PROPERTY: &'static str = "y";
pub const Z_PROPERTY: &'static str = "z";
pub const RED_PROPERTY: &'static str = "red";
pub const GREEN_PROPERTY: &'static str = "green";
pub const BLUE_PROPERTY: &'static str = "blue";
pub const ALPHA_PROPERTY: &'static str = "alpha";
pub const NX_PROPERTY: &'static str = "nx";
pub const NY_PROPERTY: &'static str = "ny";
pub const NZ_PROPERTY: &'static str = "nz";
pub const TEXTURE_U_PROPERTY: &'static str = "texture_u";
pub const TEXTURE_V_PROPERTY: &'static str = "texture_v";
pub const VERTEX_INDICES_LIST_PROPERTY: &'static str = "vertex_indices";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ElementId(pub(crate) usize);

impl Into<usize> for ElementId {
    fn into(self) -> usize {
        self.0
    }
}

impl From<usize> for ElementId {
    fn from(value: usize) -> Self {
        ElementId(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PropertyId(pub(crate) usize);

impl Into<usize> for PropertyId {
    fn into(self) -> usize {
        self.0
    }
}

impl From<usize> for PropertyId {
    fn from(value: usize) -> Self {
        PropertyId(value)
    }
}

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
pub enum PropertyDescriptor {
    Scalar {
        data_type: DataType,
        name: String,
        comments: Vec<CommentDescriptor>,
        obj_info: Vec<ObjInfoDescriptor>,
    },
    List {
        count_type: CountType,
        data_type: DataType,
        name: String,
        comments: Vec<CommentDescriptor>,
        obj_info: Vec<ObjInfoDescriptor>,
    },
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct ElementDescriptor {
    pub name: String,
    pub count: usize,
    pub properties: BTreeMap<PropertyId, PropertyDescriptor>,
    pub comments: Vec<CommentDescriptor>,
    pub obj_info: Vec<ObjInfoDescriptor>,
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary, PartialEq))]
#[derive(Debug, Clone)]
pub struct CommentDescriptor(pub String);

impl std::fmt::Display for CommentDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary, PartialEq))]
#[derive(Debug, Clone)]
pub struct ObjInfoDescriptor(pub String);

impl std::fmt::Display for ObjInfoDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct PlyDescriptor {
    pub format_type: FormatType,
    pub elements: BTreeMap<ElementId, ElementDescriptor>,
    pub comments: Vec<CommentDescriptor>,
    pub obj_info: Vec<ObjInfoDescriptor>,
}

impl Default for PlyDescriptor {
    fn default() -> Self {
        PlyDescriptor {
            format_type: FormatType::Ascii,
            elements: BTreeMap::default(),
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

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct Ply<V, I> {
    pub descriptor: PlyDescriptor,
    pub data: BTreeMap<ElementId, BTreeMap<PropertyId, Either<Vec<V>, Vec<Vec<I>>>>>,
}

impl<V, I> Ply<V, I> {
    pub fn property_id(&self, element_name: &str, property_name: &str) -> Option<(ElementId, PropertyId)> {
        self.descriptor
            .elements
            .iter()
            .filter(|(_, e)| e.name == element_name)
            .flat_map(|(e_id, e)| e.properties.iter().map(|(p_id, p)| (*e_id, p_id, p)))
            .filter(|(_, _, p)| match p {
                PropertyDescriptor::Scalar { name, .. } => name == property_name,
                PropertyDescriptor::List { name, .. } => name == property_name,
            })
            .map(|(e_id, p_id, _)| (e_id, *p_id))
            .next()
    }

    pub fn property_data(&self, element_name: &str, property_name: &str) -> Option<&Either<Vec<V>, Vec<Vec<I>>>> {
        let (e_id, p_id) = self.property_id(element_name, property_name)?;

        self.data.get(&e_id).and_then(|e| e.get(&p_id))
    }

    pub fn face_type(&self) -> Option<Primitive> {
        let d = self
            .property_data(FACE_ELEMENT, VERTEX_INDICES_LIST_PROPERTY)
            .and_then(|d| d.as_ref().right())?;

        let primitives: HashSet<usize> = d.iter().map(|i| i.len()).collect();

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
            data: BTreeMap::<ElementId, BTreeMap<PropertyId, Either<Vec<f32>, Vec<Vec<u16>>>>>::default(),
        };
    }

    #[test]
    fn ply_descriptor_has_the_following_structure() {
        let _ = PlyDescriptor {
            format_type: FormatType::Ascii,
            elements: BTreeMap::<ElementId, ElementDescriptor>::new(),
            comments: Vec::<CommentDescriptor>::new(),
            obj_info: Vec::<ObjInfoDescriptor>::new(),
        };
    }

    #[test]
    fn element_descriptor_has_the_following_structure() {
        let _ = ElementDescriptor {
            name: String::from("vertex"),
            count: 0usize,
            properties: BTreeMap::<PropertyId, PropertyDescriptor>::new(),
            comments: Vec::<CommentDescriptor>::new(),
            obj_info: Vec::<ObjInfoDescriptor>::new(),
        };
    }

    #[test]
    fn property_descriptor_has_the_following_structure() {
        let _ = PropertyDescriptor::Scalar {
            name: String::from("x"),
            data_type: DataType::F32,
            comments: Vec::<CommentDescriptor>::new(),
            obj_info: Vec::<ObjInfoDescriptor>::new(),
        };
        let _ = PropertyDescriptor::List {
            name: String::from("i"),
            count_type: CountType::U16,
            data_type: DataType::F32,
            comments: Vec::<CommentDescriptor>::new(),
            obj_info: Vec::<ObjInfoDescriptor>::new(),
        };
    }
}
