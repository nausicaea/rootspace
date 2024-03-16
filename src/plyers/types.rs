use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use clap::builder::PossibleValue;

pub const VERTEX_ELEMENT: &str = "vertex";
pub const FACE_ELEMENT: &str = "face";
pub const X_PROPERTY: &str = "x";
pub const Y_PROPERTY: &str = "y";
pub const Z_PROPERTY: &str = "z";
pub const NX_PROPERTY: &str = "nx";
pub const NY_PROPERTY: &str = "ny";
pub const NZ_PROPERTY: &str = "nz";
pub const TEXTURE_U_PROPERTY: &str = "texture_u";
pub const TEXTURE_V_PROPERTY: &str = "texture_v";
pub const S_PROPERTY: &str = "s";
pub const T_PROPERTY: &str = "t";
pub const U_PROPERTY: &str = "u";
pub const V_PROPERTY: &str = "v";
pub const VERTEX_INDICES_LIST_PROPERTY: &str = "vertex_indices";

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the sequence of values contains inconsistent data types")]
pub struct InconsistentDataTypes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ElementId(pub usize);

impl From<ElementId> for usize {
    fn from(val: ElementId) -> Self {
        val.0
    }
}

impl From<usize> for ElementId {
    fn from(value: usize) -> Self {
        ElementId(value)
    }
}

impl std::fmt::Display for ElementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PropertyId(pub usize);

impl From<PropertyId> for usize {
    fn from(val: PropertyId) -> Self {
        val.0
    }
}

impl From<usize> for PropertyId {
    fn from(value: usize) -> Self {
        PropertyId(value)
    }
}

impl std::fmt::Display for PropertyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormatType {
    Ascii,
    BinaryLittleEndian,
    BinaryBigEndian,
}

impl clap::ValueEnum for FormatType {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            FormatType::Ascii,
            FormatType::BinaryLittleEndian,
            FormatType::BinaryBigEndian,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            FormatType::Ascii => Some(PossibleValue::new("ascii")),
            FormatType::BinaryLittleEndian => Some(PossibleValue::new("binary_little_endian")),
            FormatType::BinaryBigEndian => Some(PossibleValue::new("binary_big_endian")),
        }
    }
}

impl Display for FormatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatType::Ascii => write!(f, "ascii"),
            FormatType::BinaryLittleEndian => write!(f, "binary_little_endian"),
            FormatType::BinaryBigEndian => write!(f, "binary_big_endian"),
        }
    }
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CountType {
    U8,
    U16,
    U32,
    U64,
}

impl Display for CountType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CountType::U8 => write!(f, "uint8"),
            CountType::U16 => write!(f, "uint16"),
            CountType::U32 => write!(f, "uint32"),
            CountType::U64 => write!(f, "uint64"),
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
    I64,
    U64,
    F32,
    F64,
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::I8 => write!(f, "int8"),
            DataType::U8 => write!(f, "uint8"),
            DataType::I16 => write!(f, "int16"),
            DataType::U16 => write!(f, "uint16"),
            DataType::I32 => write!(f, "int32"),
            DataType::U32 => write!(f, "uint32"),
            DataType::I64 => write!(f, "int64"),
            DataType::U64 => write!(f, "uint64"),
            DataType::F32 => write!(f, "float32"),
            DataType::F64 => write!(f, "float64"),
        }
    }
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

impl PropertyDescriptor {
    pub fn comments(&self) -> impl Iterator<Item = &CommentDescriptor> {
        match self {
            PropertyDescriptor::Scalar { comments, .. } => comments.iter(),
            PropertyDescriptor::List { comments, .. } => comments.iter(),
        }
    }

    pub fn obj_info(&self) -> impl Iterator<Item = &ObjInfoDescriptor> {
        match self {
            PropertyDescriptor::Scalar { obj_info, .. } => obj_info.iter(),
            PropertyDescriptor::List { obj_info, .. } => obj_info.iter(),
        }
    }
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

impl AsRef<str> for CommentDescriptor {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for CommentDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg_attr(test, derive(proptest_derive::Arbitrary, PartialEq))]
#[derive(Debug, Clone)]
pub struct ObjInfoDescriptor(pub String);

impl AsRef<str> for ObjInfoDescriptor {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

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
    Single,
    Lines,
    Triangles,
    Quads,
    Mixed,
}

impl From<usize> for Primitive {
    fn from(value: usize) -> Self {
        use self::Primitive::*;
        match value {
            1 => Single,
            2 => Lines,
            3 => Triangles,
            4 => Quads,
            _ => Mixed,
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("mixed primitives are ambiguous")]
pub struct AmbiguousMixedPrimitive;

impl TryFrom<Primitive> for usize {
    type Error = AmbiguousMixedPrimitive;

    fn try_from(value: Primitive) -> Result<Self, Self::Error> {
        use self::Primitive::*;
        match value {
            Single => Ok(1),
            Lines => Ok(2),
            Triangles => Ok(3),
            Quads => Ok(4),
            Mixed => Err(AmbiguousMixedPrimitive),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
}

macro_rules! impl_from_and_try_into {
    ($prim:ty, $var:ident) => {
        impl From<$prim> for Value {
            fn from(value: $prim) -> Self {
                Self::$var(value)
            }
        }

        impl TryInto<$prim> for Value {
            type Error = ();

            fn try_into(self) -> Result<$prim, Self::Error> {
                match self {
                    Self::$var(n) => Ok(n),
                    _ => Err(()),
                }
            }
        }
    };
}

impl_from_and_try_into!(u8, U8);
impl_from_and_try_into!(i8, I8);
impl_from_and_try_into!(u16, U16);
impl_from_and_try_into!(i16, I16);
impl_from_and_try_into!(u32, U32);
impl_from_and_try_into!(i32, I32);
impl_from_and_try_into!(u64, U64);
impl_from_and_try_into!(i64, I64);
impl_from_and_try_into!(f32, F32);
impl_from_and_try_into!(f64, F64);

#[derive(Debug, Clone, PartialEq)]
pub enum Values {
    U8(Vec<u8>),
    I8(Vec<i8>),
    U16(Vec<u16>),
    I16(Vec<i16>),
    U32(Vec<u32>),
    I32(Vec<i32>),
    U64(Vec<u64>),
    I64(Vec<i64>),
    F32(Vec<f32>),
    F64(Vec<f64>),
}

impl Values {
    pub fn with_data_type(dt: DataType) -> Self {
        match dt {
            DataType::U8 => Values::U8(Vec::new()),
            DataType::I8 => Values::I8(Vec::new()),
            DataType::U16 => Values::U16(Vec::new()),
            DataType::I16 => Values::I16(Vec::new()),
            DataType::U32 => Values::U32(Vec::new()),
            DataType::I32 => Values::I32(Vec::new()),
            DataType::U64 => Values::U64(Vec::new()),
            DataType::I64 => Values::I64(Vec::new()),
            DataType::F32 => Values::F32(Vec::new()),
            DataType::F64 => Values::F64(Vec::new()),
        }
    }

    pub fn try_push(&mut self, v: Value) -> Result<(), InconsistentDataTypes> {
        match (self, v) {
            (Values::U8(acc), Value::U8(v)) => acc.push(v),
            (Values::I8(acc), Value::I8(v)) => acc.push(v),
            (Values::U16(acc), Value::U16(v)) => acc.push(v),
            (Values::I16(acc), Value::I16(v)) => acc.push(v),
            (Values::U32(acc), Value::U32(v)) => acc.push(v),
            (Values::I32(acc), Value::I32(v)) => acc.push(v),
            (Values::U64(acc), Value::U64(v)) => acc.push(v),
            (Values::I64(acc), Value::I64(v)) => acc.push(v),
            (Values::F32(acc), Value::F32(v)) => acc.push(v),
            (Values::F64(acc), Value::F64(v)) => acc.push(v),
            _ => return Err(InconsistentDataTypes),
        }

        Ok(())
    }

    pub fn try_extend<I>(&mut self, iter: I) -> Result<(), InconsistentDataTypes>
    where
        I: IntoIterator<Item = Value>,
    {
        for value in iter.into_iter() {
            self.try_push(value)?;
        }

        Ok(())
    }
}

pub trait AsSlice<T> {
    fn as_slice(&self) -> Option<&[T]>;
}

macro_rules! impl_as_slice {
    ($ty:ty, $var:ident) => {
        impl AsSlice<$ty> for Values {
            fn as_slice(&self) -> Option<&[$ty]> {
                match self {
                    Values::$var(v) => Some(v),
                    _ => None,
                }
            }
        }
    };
}

impl_as_slice!(u8, U8);
impl_as_slice!(i8, I8);
impl_as_slice!(u16, U16);
impl_as_slice!(i16, I16);
impl_as_slice!(u32, U32);
impl_as_slice!(i32, I32);
impl_as_slice!(u64, U64);
impl_as_slice!(i64, I64);
impl_as_slice!(f32, F32);
impl_as_slice!(f64, F64);

impl TryFrom<(DataType, Vec<Value>)> for Values {
    type Error = InconsistentDataTypes;

    fn try_from(value: (DataType, Vec<Value>)) -> Result<Self, Self::Error> {
        let mut accum = Values::with_data_type(value.0);
        accum.try_extend(value.1)?;
        Ok(accum)
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct Ply {
    pub descriptor: PlyDescriptor,
    pub data: BTreeMap<PropertyId, (Primitive, Values)>,
}

impl Ply {
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

    pub fn primitive(&self) -> Option<Primitive> {
        let p_id = self
            .property_id(FACE_ELEMENT, VERTEX_INDICES_LIST_PROPERTY)
            .map(|(_, p_id)| p_id)?;

        Some(self.data[&p_id].0)
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
            data: BTreeMap::<PropertyId, (Primitive, Values)>::default(),
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
