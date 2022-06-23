use std::io::Read;

use super::{
    generic_parsers::{parse_from_lut, parse_unsigned_from_ascii},
    tables::{CNT_TYP_TBL, DAT_TYP_TBL, FMT_TYP_TBL},
    types::{CountType, DataType, FormatType},
    Error,
};
use crate::{generic_parsers::{parse_string_from_ascii, parse_word_from_ascii, parse_whitespace}, tables::PLY_HEADER_START};

pub fn parse_begin_header<R>(file: &mut R) -> Result<(), Error>
where
    R: Read,
{
    parse_from_lut(file, &[&PLY_HEADER_START], |_| Ok(()))
}

pub fn parse_format<R>(file: &mut R) -> Result<(FormatType, String), Error>
where
    R: Read,
{
    let format_type = parse_from_lut(file, FMT_TYP_TBL, |k| match k {
        0 => {
            return Ok(FormatType::Ascii);
        }
        1 => {
            return Ok(FormatType::BinaryLittleEndian);
        }
        2 => {
            return Ok(FormatType::BinaryBigEndian);
        }
        _ => unreachable!(),
    })?;

    parse_whitespace(file)?;

    let format_version = parse_string_from_ascii(file)?;

    Ok((format_type, format_version))
}

pub fn parse_element<R>(file: &mut R) -> Result<(String, usize), Error>
where
    R: Read,
{
    let name = parse_word_from_ascii(file)?;

    parse_whitespace(file)?;

    let count = parse_unsigned_from_ascii(file)?;

    Ok((name, count))
}

pub fn parse_property<R>(file: &mut R) -> Result<(DataType, String), Error>
where
    R: Read,
{
    let data_type = parse_from_lut(file, DAT_TYP_TBL, |k| match k {
        0 => Ok(DataType::I8),
        1 => Ok(DataType::U8),
        2 => Ok(DataType::I16),
        3 => Ok(DataType::U16),
        4 => Ok(DataType::I32),
        5 => Ok(DataType::U32),
        6 => Ok(DataType::F32),
        7 => Ok(DataType::F64),
        8 => Ok(DataType::I8),
        9 => Ok(DataType::U8),
        10 => Ok(DataType::I16),
        11 => Ok(DataType::U16),
        12 => Ok(DataType::I32),
        13 => Ok(DataType::U32),
        14 => Ok(DataType::F32),
        15 => Ok(DataType::F64),
        _ => unreachable!(),
    })?;

    parse_whitespace(file)?;

    let name = parse_word_from_ascii(file)?;

    Ok((data_type, name))
}

pub fn parse_list_property<R>(file: &mut R) -> Result<(CountType, DataType, String), Error>
where
    R: Read,
{
    let count_type = parse_from_lut(file, CNT_TYP_TBL, |k| match k {
        0|3 => Ok(CountType::U8),
        2|5 => Ok(CountType::U16),
        1|4 => Ok(CountType::U32),
        _ => unreachable!(),
    })?;

    parse_whitespace(file)?;

    let data_type = parse_from_lut(file, DAT_TYP_TBL, |k| match k {
        0|7 => Ok(DataType::I8),
        10|13 => Ok(DataType::U8),
        6|9 => Ok(DataType::I16),
        12|15 => Ok(DataType::U16),
        5|8 => Ok(DataType::I32),
        11|14 => Ok(DataType::U32),
        3|4 => Ok(DataType::F32),
        1|2 => Ok(DataType::F64),
        _ => unreachable!(),
    })?;

    parse_whitespace(file)?;

    let name = parse_word_from_ascii(file)?;

    Ok((count_type, data_type, name))
}

pub fn parse_comment<R>(file: &mut R) -> Result<String, Error>
where
    R: Read,
{
    let comment = parse_string_from_ascii(file)?;

    Ok(comment)
}

pub fn parse_obj_info<R>(file: &mut R) -> Result<String, Error>
where
    R: Read,
{
    let obj_info = parse_string_from_ascii(file)?;

    Ok(obj_info)
}

pub fn parse_data_point<R>(
    _file: &mut R,
    data_type: DataType,
    format_type: FormatType,
) -> Result<Vec<u8>, Error>
where
    R: Read,
{
    match format_type {
        FormatType::Ascii => match data_type {
            DataType::F32 | DataType::F64 => {
                todo!()
            }
            _ => {
                todo!()
            }
        },
        FormatType::BinaryLittleEndian => todo!(),
        FormatType::BinaryBigEndian => todo!(),
    }
}

pub fn parse_length<R>(
    file: &mut R,
    _count_type: CountType,
    format_type: FormatType,
) -> Result<usize, Error>
where
    R: Read,
{
    match format_type {
        FormatType::Ascii => {
            let count = parse_unsigned_from_ascii(file)?;
            Ok(count)
        }
        FormatType::BinaryLittleEndian => todo!(),
        FormatType::BinaryBigEndian => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn parse_obj_info_returns_a_string(source in r"[[:ascii:]]+\n") {
            let expected: String = source.chars().take_while(|c| c != &'\n').collect();
            let mut stream = source.as_bytes();

            let r = parse_obj_info(&mut stream);

            prop_assert!(r.is_ok(), "{}", r.unwrap_err());
            prop_assert_eq!(r.unwrap(), expected);
        }

        #[test]
        fn parse_comment_returns_a_string(source in r"[[:ascii:]]+\n") {
            let expected: String = source.chars().take_while(|c| c != &'\n').collect();
            let mut stream = source.as_bytes();

            let r = parse_comment(&mut stream);

            prop_assert!(r.is_ok(), "{}", r.unwrap_err());
            prop_assert_eq!(r.unwrap(), expected);
        }

        #[test]
        fn parse_list_property_returns_count_type_data_type_and_a_string(ct: CountType, dt: DataType, n in r"[\x00-\x09\x0b-\xff]+") {
            let source = format!("{} {} {}\n", ct, dt, &n);
            let mut stream = source.as_bytes();

            let r = parse_list_property(&mut stream);

            prop_assert!(r.is_ok(), "{}", r.unwrap_err());
            prop_assert_eq!(r.unwrap(), (ct, dt, n));
        }
    }
}
