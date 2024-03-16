use std::io::Write;

use num_traits::ToBytes;

use crate::plyers::{
    types::{PlyDescriptor, Primitive, PropertyDescriptor},
    PlyError,
};

pub fn write_header<W: Write>(f: &mut W, descriptor: &PlyDescriptor) -> Result<(), PlyError> {
    writeln!(f, "ply")?;
    writeln!(f, "format {} 1.0", descriptor.format_type)?;
    for comment in &descriptor.comments {
        writeln!(f, "comment {}", &comment.0)?;
    }
    for obj_info in &descriptor.obj_info {
        writeln!(f, "obj_info {}", &obj_info.0)?;
    }
    for element in descriptor.elements.values() {
        for comment in &element.comments {
            writeln!(f, "comment {}", &comment.0)?;
        }
        for obj_info in &element.obj_info {
            writeln!(f, "obj_info {}", &obj_info.0)?;
        }
        writeln!(f, "element {} {}", element.name, element.count)?;
        for property in element.properties.values() {
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

pub fn write_le_values<const N: usize, W: Write, T: ToBytes<Bytes = [u8; N]>>(
    f: &mut W,
    primitive: &Primitive,
    descriptor: &PropertyDescriptor,
    values: &[T],
    element_index: usize,
) -> Result<(), PlyError> {
    match primitive {
        Primitive::Single => match descriptor {
            PropertyDescriptor::Scalar { .. } => {
                f.write_all(&values[element_index].to_le_bytes()[..])?;
            }
            PropertyDescriptor::List { .. } => {
                f.write_all(&[1u8])?;
                f.write_all(&values[element_index].to_le_bytes()[..])?;
            }
        },
        p => {
            write_le_lists(f, *p, values, element_index)?;
        }
    }

    Ok(())
}

pub fn write_be_values<const N: usize, W: Write, T: ToBytes<Bytes = [u8; N]>>(
    f: &mut W,
    primitive: &Primitive,
    descriptor: &PropertyDescriptor,
    values: &[T],
    element_index: usize,
) -> Result<(), PlyError> {
    match primitive {
        Primitive::Single => match descriptor {
            PropertyDescriptor::Scalar { .. } => {
                f.write_all(&values[element_index].to_be_bytes()[..])?;
            }
            PropertyDescriptor::List { .. } => {
                f.write_all(&[1u8])?;
                f.write_all(&values[element_index].to_be_bytes()[..])?;
            }
        },
        p => {
            write_be_lists(f, *p, values, element_index)?;
        }
    }

    Ok(())
}

pub fn write_ascii_values<W: Write, T: std::fmt::Display>(
    f: &mut W,
    primitive: &Primitive,
    descriptor: &PropertyDescriptor,
    values: &[T],
    element_index: usize,
    is_last_property: bool,
) -> Result<(), PlyError> {
    let normal_sep = " ";
    let trailing_sep = if is_last_property { "\n" } else { " " };
    match primitive {
        Primitive::Single => match descriptor {
            PropertyDescriptor::Scalar { .. } => {
                write!(f, "{}{}", &values[element_index], trailing_sep)?;
            }
            PropertyDescriptor::List { .. } => {
                write!(f, "1{}{}{}", normal_sep, &values[element_index], trailing_sep)?;
            }
        },
        p => {
            write_ascii_lists(f, *p, values, element_index, normal_sep, trailing_sep)?;
        }
    }

    Ok(())
}

fn write_le_lists<const N: usize, W: Write, T: ToBytes<Bytes = [u8; N]>>(
    f: &mut W,
    primitive: Primitive,
    values: &[T],
    element_index: usize,
) -> Result<(), PlyError> {
    let stride: usize = primitive.try_into()?;
    let value_chunk = values[(stride * element_index)..(stride * (element_index + 1))]
        .iter()
        .flat_map(|v| v.to_le_bytes().into_iter())
        .collect::<Vec<_>>();
    f.write_all(&[stride as u8])?;
    f.write_all(&value_chunk)?;

    Ok(())
}

fn write_be_lists<const N: usize, W: Write, T: ToBytes<Bytes = [u8; N]>>(
    f: &mut W,
    primitive: Primitive,
    values: &[T],
    element_index: usize,
) -> Result<(), PlyError> {
    let stride: usize = primitive.try_into()?;
    let value_chunk = values[(stride * element_index)..(stride * (element_index + 1))]
        .iter()
        .flat_map(|v| v.to_be_bytes().into_iter())
        .collect::<Vec<_>>();
    f.write_all(&[stride as u8])?;
    f.write_all(&value_chunk)?;

    Ok(())
}

fn write_ascii_lists<W: Write, T: std::fmt::Display>(
    f: &mut W,
    primitive: Primitive,
    values: &[T],
    element_index: usize,
    normal_sep: &str,
    trailing_sep: &str,
) -> Result<(), PlyError> {
    let stride: usize = primitive.try_into()?;
    write!(f, "{}{}", stride, normal_sep)?;
    let chunk_values = values[(stride * element_index)..(stride * (element_index + 1))]
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(normal_sep);
    write!(f, "{}{}", chunk_values, trailing_sep)?;

    Ok(())
}
