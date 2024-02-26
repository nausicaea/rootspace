use crate::plyers::types::{PlyDescriptor, Primitive, PropertyDescriptor};
use crate::plyers::PlyError;
use num_traits::{One, ToBytes};
use std::io::Write;
use std::ops::Add;

pub fn write_header<W: Write>(f: &mut W, descriptor: &PlyDescriptor) -> Result<(), PlyError> {
    writeln!(f, "ply")?;
    writeln!(f, "format {} 1.0", descriptor.format_type)?;
    for comment in &descriptor.comments {
        writeln!(f, "comment {}", &comment.0)?;
    }
    for obj_info in &descriptor.obj_info {
        writeln!(f, "obj_info {}", &obj_info.0)?;
    }
    for (_, element) in &descriptor.elements {
        for comment in &element.comments {
            writeln!(f, "comment {}", &comment.0)?;
        }
        for obj_info in &element.obj_info {
            writeln!(f, "obj_info {}", &obj_info.0)?;
        }
        writeln!(f, "element {} {}", element.name, element.count)?;
        for (_, property) in &element.properties {
            for comment in property.comments() {
                writeln!(f, "comment {}", &comment.0)?;
            }
            for obj_info in property.obj_info() {
                writeln!(f, "obj_info {}", &obj_info.0)?;
            }
            match property {
                PropertyDescriptor::Scalar { data_type, name, .. } => {
                    writeln!(f, "property {} {}", data_type, name)?;
                }
                PropertyDescriptor::List {
                    count_type,
                    data_type,
                    name,
                    ..
                } => {
                    writeln!(f, "property list {} {} {}", count_type, data_type, name)?;
                }
            }
        }
    }
    writeln!(f, "end_header")?;

    Ok(())
}

pub fn write_values_ascii<W: Write, T: ToString + num_traits::One + Add<T, Output = T>>(
    f: &mut W,
    primitive: &Primitive,
    values: &[T],
) -> Result<(), PlyError> {
    match primitive {
        Primitive::Single => writeln!(
            f,
            "{}",
            values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ")
        )?,
        Primitive::Triangles => {
            let three = T::one() + T::one() + T::one();
            let chunk_iter = values.chunks_exact(3);
            if !chunk_iter.remainder().is_empty() {
                return Err(PlyError::DataError(String::from(
                    "property data count is not divisible by three",
                )));
            }
            for chunk in chunk_iter {
                let chunk = std::iter::once(&three)
                    .chain(chunk.iter())
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                writeln!(f, "{}", chunk)?;
            }
        }
        Primitive::Quads => {
            let four = T::one() + T::one() + T::one() + T::one();
            let chunk_iter = values.chunks_exact(4);
            if !chunk_iter.remainder().is_empty() {
                return Err(PlyError::DataError(String::from(
                    "property data count is not divisible by four",
                )));
            }
            for chunk in chunk_iter {
                let chunk = std::iter::once(&four)
                    .chain(chunk.iter())
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                writeln!(f, "{}", chunk)?;
            }
        }
        _ => unimplemented!("cannot write data with mixed primitives"),
    }
    Ok(())
}

pub fn write_values_le<
    const N: usize,
    W: Write,
    T: num_traits::ToBytes<Bytes = [u8; N]> + num_traits::One + Add<T, Output = T>,
>(
    f: &mut W,
    primitive: &Primitive,
    values: &[T],
) -> Result<(), PlyError> {
    match primitive {
        Primitive::Single => {
            let mut values_u8: Vec<u8> = Vec::with_capacity(values.len() * std::mem::size_of::<T>());
            for v in values {
                values_u8.extend_from_slice(&v.to_le_bytes()[..]);
            }
            f.write(&values_u8)?;
        }
        Primitive::Triangles => {
            let chunk_iter = values.chunks_exact(3);
            if !chunk_iter.remainder().is_empty() {
                return Err(PlyError::DataError(String::from(
                    "property data count is not divisible by three",
                )));
            }
            for chunk in chunk_iter {
                let mut chunk_u8: Vec<u8> = Vec::with_capacity(1 + chunk.len() * std::mem::size_of::<T>());
                chunk_u8.push(3u8);
                for v in chunk {
                    chunk_u8.extend_from_slice(&v.to_le_bytes()[..]);
                }
                f.write(&chunk_u8)?;
            }
        }
        Primitive::Quads => {
            let chunk_iter = values.chunks_exact(4);
            if !chunk_iter.remainder().is_empty() {
                return Err(PlyError::DataError(String::from(
                    "property data count is not divisible by four",
                )));
            }
            for chunk in chunk_iter {
                let mut chunk_u8: Vec<u8> = Vec::with_capacity(1 + chunk.len() * std::mem::size_of::<T>());
                chunk_u8.push(4u8);
                for v in chunk {
                    chunk_u8.extend_from_slice(&v.to_le_bytes()[..]);
                }
                f.write(&chunk_u8)?;
            }
        }
        _ => unimplemented!("cannot write data with mixed primitives"),
    }

    Ok(())
}

pub fn write_values_be<
    const N: usize,
    W: Write,
    T: num_traits::ToBytes<Bytes = [u8; N]> + num_traits::One + Add<T, Output = T>,
>(
    f: &mut W,
    primitive: &Primitive,
    values: &[T],
) -> Result<(), PlyError> {
    match primitive {
        Primitive::Single => {
            let mut values_u8: Vec<u8> = Vec::with_capacity(values.len() * std::mem::size_of::<T>());
            for v in values {
                values_u8.extend_from_slice(&v.to_be_bytes()[..]);
            }
            f.write(&values_u8)?;
        }
        Primitive::Triangles => {
            let chunk_iter = values.chunks_exact(3);
            if !chunk_iter.remainder().is_empty() {
                return Err(PlyError::DataError(String::from(
                    "property data count is not divisible by three",
                )));
            }
            for chunk in chunk_iter {
                let mut chunk_u8: Vec<u8> = Vec::with_capacity(1 + chunk.len() * std::mem::size_of::<T>());
                chunk_u8.push(3u8);
                for v in chunk {
                    chunk_u8.extend_from_slice(&v.to_be_bytes()[..]);
                }
                f.write(&chunk_u8)?;
            }
        }
        Primitive::Quads => {
            let chunk_iter = values.chunks_exact(4);
            if !chunk_iter.remainder().is_empty() {
                return Err(PlyError::DataError(String::from(
                    "property data count is not divisible by four",
                )));
            }
            for chunk in chunk_iter {
                let mut chunk_u8: Vec<u8> = Vec::with_capacity(1 + chunk.len() * std::mem::size_of::<T>());
                chunk_u8.push(4u8);
                for v in chunk {
                    chunk_u8.extend_from_slice(&v.to_be_bytes()[..]);
                }
                f.write(&chunk_u8)?;
            }
        }
        _ => unimplemented!("cannot write data with mixed primitives"),
    }

    Ok(())
}
