use std::num::{ParseIntError, ParseFloatError};

mod common;
mod header;

#[derive(Debug, thiserror::Error)]
pub enum ParseNumError {
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
}

#[cfg(test)]
mod tests {
    use nom::{
        combinator::{all_consuming, flat_map, map_res, map},
        error::ParseError,
        sequence::{terminated, pair}, multi::{fold_many_m_n, length_count},
        bytes::complete::take_till1,
        IResult,
    };

    use super::*;
    use super::header::header;
    use super::common::{is_whitespace, whitespace};
    use crate::types::{DataType, ElementDescriptor, FormatType, Ply};

    #[test]
    fn playground() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
        let (rest, descriptor) = header(&input[..]).unwrap();

        fn ascii_usize(input: &[u8]) -> IResult<&[u8], usize> {
            map_res(
                terminated(take_till1(is_whitespace), whitespace),
                |cd| {
                    let cd = std::str::from_utf8(cd)?;
                    let cd = cd.parse::<usize>()?;
                    Result::<_, ParseNumError>::Ok(cd)
                },
            )(input)
        }

        fn ascii_number_fct<'a>(data_type: DataType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>> {
            map_res(
                terminated(take_till1(is_whitespace), whitespace),
                move |pd| {
                    let pd = std::str::from_utf8(pd)?;
                    let pd = match data_type {
                        DataType::U8 => pd
                            .parse::<u8>()
                            .map(|pd| pd.to_ne_bytes().into_iter().collect::<Vec<_>>())?,
                        DataType::I8 => pd
                            .parse::<i8>()
                            .map(|pd| pd.to_ne_bytes().into_iter().collect::<Vec<_>>())?,
                        DataType::U16 => pd
                            .parse::<u16>()
                            .map(|pd| pd.to_ne_bytes().into_iter().collect::<Vec<_>>())?,
                        DataType::I16 => pd
                            .parse::<i16>()
                            .map(|pd| pd.to_ne_bytes().into_iter().collect::<Vec<_>>())?,
                        DataType::U32 => pd
                            .parse::<u32>()
                            .map(|pd| pd.to_ne_bytes().into_iter().collect::<Vec<_>>())?,
                        DataType::I32 => pd
                            .parse::<i32>()
                            .map(|pd| pd.to_ne_bytes().into_iter().collect::<Vec<_>>())?,
                        DataType::F32 => pd
                            .parse::<f32>()
                            .map(|pd| pd.to_ne_bytes().into_iter().collect::<Vec<_>>())?,
                        DataType::F64 => pd
                            .parse::<f64>()
                            .map(|pd| pd.to_ne_bytes().into_iter().collect::<Vec<_>>())?,
                    };

                    Result::<_, ParseNumError>::Ok(pd)
                },
            )
        }

        fn ascii_properties_fct<'a>(data_types: Vec<DataType>) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>> {
            let m = data_types.len();
            let mut i = 0;

            fold_many_m_n(m, m, move |input| {
                    let r = ascii_number_fct(data_types[i])(input);
                    i += 1;
                    r
                },
                Vec::new,
                |mut p_acc, p| {
                    p_acc.extend(p);
                    p_acc
                },
            )
        }

        fn ascii_list_properties_fct<'a>(data_types: Vec<DataType>) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>> {
            let m = data_types.len();
            let mut i = 0;

            fold_many_m_n(m, m, move |input| {
                    let r = length_count(ascii_usize, ascii_number_fct(data_types[i]))(input);
                    i += 1;
                    r
                },
                Vec::new,
                |mut pl_acc, pl| {
                    pl_acc.extend(pl.into_iter().flatten());
                    pl_acc
                },
            )
        }

        fn ascii_elements_fct<'a>(elements: Vec<ElementDescriptor>) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], (Vec<u8>, Vec<u8>)> {
            let m = elements.len();
            let mut i = 0;

            fold_many_m_n(m, m, move |input| {
                    let el = &elements[i];
                    let r = pair(
                        ascii_properties_fct(el.properties.iter().map(|p| p.data_type).collect()),
                        ascii_list_properties_fct(el.list_properties.iter().map(|p| p.data_type).collect())
                    )(input);
                    i += 1;
                    r
                },
                || (Vec::new(), Vec::new()),
                |(mut p_acc, mut pl_acc), (p, pl)| {
                    p_acc.extend(p);
                    pl_acc.extend(pl);

                    (p_acc, pl_acc)
                },
            )
        }

        fn body_factory<'a, E: ParseError<&'a [u8]>>(
            format_type: FormatType,
            elements: &[ElementDescriptor],
        ) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], (Vec<u8>, Vec<u8>), E> {
            let mut property_data: Vec<u8> = Vec::new();
            let mut list_property_data: Vec<u8> = Vec::new();

            match format_type {
                FormatType::Ascii => {
                    let mut ascii_parsers = Vec::new();
                    for element in elements {
                        for property in &element.properties {
                            let property_value = ply::ascii_number_parser(property.data_type).parse(stream)?;
                            property_data.extend(property_value);
                        }

                        for list_property in &element.list_properties {
                            let count = ply::ascii_usize_parser().parse(stream)?;
                            let property_values =
                                repeat_exact(ply::ascii_number_parser(list_property.data_type), count).parse(stream)?;
                            for property_value in property_values {
                                list_property_data.extend(property_value);
                            }
                        }
                    }
                }
                FormatType::BinaryLittleEndian => {
                    for element in elements {
                        for property in &element.properties {
                            let property_value = le_number::le_number(property.data_type).parse(stream)?;
                            property_data.extend(property_value);
                        }

                        for list_property in &element.list_properties {
                            let count = le_count::le_count(list_property.count_type).parse(stream)?;
                            let property_values =
                                repeat_exact(le_number::le_number(list_property.data_type), count).parse(stream)?;
                            for property_value in property_values {
                                list_property_data.extend(property_value);
                            }
                        }
                    }
                }
                FormatType::BinaryBigEndian => {
                    for element in elements {
                        for property in &element.properties {
                            let property_value = be_number::be_number(property.data_type).parse(stream)?;
                            property_data.extend(property_value);
                        }

                        for list_property in &element.list_properties {
                            let count = be_count::be_count(list_property.count_type).parse(stream)?;
                            let property_values =
                                repeat_exact(be_number::be_number(list_property.data_type), count).parse(stream)?;
                            for property_value in property_values {
                                list_property_data.extend(property_value);
                            }
                        }
                    }
                }
            }
        }

        fn all_of_it(input: &[u8]) -> IResult<&[u8], Ply> {
            all_consuming(flat_map(header, |descriptor| {
                map(
                    body_factory(descriptor.format_type, &descriptor.elements),
                    |(property_data, list_property_data)| Ply {
                        descriptor,
                        property_data,
                        list_property_data,
                    },
                )
            }))(input)
        }
    }
}
