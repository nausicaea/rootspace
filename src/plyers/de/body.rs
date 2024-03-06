use log::{debug, error, trace};
use std::collections::BTreeMap;

use nom::{
    bytes::complete::take_till1,
    combinator::{map, map_res},
    error::{context, ContextError, FromExternalError, ParseError},
    multi::length_count,
    number::complete::{
        be_f32, be_f64, be_i16, be_i32, be_i64, be_i8, be_u16, be_u32, be_u64, be_u8, le_f32, le_f64, le_i16, le_i32,
        le_i64, le_i8, le_u16, le_u32, le_u64, le_u8, recognize_float,
    },
    sequence::terminated,
    IResult,
};

use super::super::types::{
    CountType, DataType, ElementDescriptor, ElementId, FormatType, PlyDescriptor, Primitive, PropertyDescriptor,
    PropertyId, Value, Values,
};
use super::{
    common::{fold_exact, is_whitespace, whitespace},
    ParseNumError,
};

fn ascii_count_fct<'a, E>(_count_type: CountType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>
where
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]>,
{
    context(
        "plyers::de::body::ascii_count_fct",
        map_res(terminated(take_till1(is_whitespace), whitespace), |cd| {
            trace!("Parsing ASCII count data as utf8: {:?}", cd);
            let cd = std::str::from_utf8(cd)?;
            trace!("Parsing ASCII count data as usize: {:?}", cd);
            let cd = cd.parse::<usize>().map_err(|e| {
                error!("Expected a usize, got {}: {}", cd, e);
                e
            })?;
            Result::<_, ParseNumError>::Ok(cd)
        }),
    )
}

fn ascii_number_fct<'a, E>(data_type: DataType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Value, E>
where
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]>,
{
    context(
        "plyers::de::body::ascii_number_fct",
        map_res(terminated(recognize_float, whitespace), move |pd| {
            trace!("Parsing ASCII property data as utf8: {:?}", pd);
            let pd = std::str::from_utf8(pd)?;
            trace!("Parsing ASCII property data as {}: {:?}", data_type, pd);
            let pd = match data_type {
                DataType::U8 => pd
                    .parse::<u8>()
                    .map_err(|e| {
                        error!("Expected a u8, got {}: {}", pd, e);
                        ParseNumError::ParseIntError(e)
                    })
                    .map(Value::from)?,
                DataType::I8 => pd
                    .parse::<i8>()
                    .map_err(|e| {
                        error!("Expected a i8, got {}: {}", pd, e);
                        ParseNumError::ParseIntError(e)
                    })
                    .map(Value::from)?,
                DataType::U16 => pd
                    .parse::<u16>()
                    .map_err(|e| {
                        error!("Expected a u16, got {}: {}", pd, e);
                        ParseNumError::ParseIntError(e)
                    })
                    .map(Value::from)?,
                DataType::I16 => pd
                    .parse::<i16>()
                    .map_err(|e| {
                        error!("Expected a i16, got {}: {}", pd, e);
                        ParseNumError::ParseIntError(e)
                    })
                    .map(Value::from)?,
                DataType::U32 => pd
                    .parse::<u32>()
                    .map_err(|e| {
                        error!("Expected a u32, got {}: {}", pd, e);
                        ParseNumError::ParseIntError(e)
                    })
                    .map(Value::from)?,
                DataType::I32 => pd
                    .parse::<i32>()
                    .map_err(|e| {
                        error!("Expected a i32, got {}: {}", pd, e);
                        ParseNumError::ParseIntError(e)
                    })
                    .map(Value::from)?,
                DataType::U64 => pd
                    .parse::<u64>()
                    .map_err(|e| {
                        error!("Expected a u64, got {}: {}", pd, e);
                        ParseNumError::ParseIntError(e)
                    })
                    .map(Value::from)?,
                DataType::I64 => pd
                    .parse::<i64>()
                    .map_err(|e| {
                        error!("Expected a i64, got {}: {}", pd, e);
                        ParseNumError::ParseIntError(e)
                    })
                    .map(Value::from)?,
                DataType::F32 => pd
                    .parse::<f32>()
                    .map_err(|e| {
                        error!("Expected a f32, got {}: {}", pd, e);
                        ParseNumError::ParseFloatError(e)
                    })
                    .map(Value::from)?,
                DataType::F64 => pd
                    .parse::<f64>()
                    .map_err(|e| {
                        error!("Expected a f64, got {}: {}", pd, e);
                        ParseNumError::ParseFloatError(e)
                    })
                    .map(Value::from)?,
            };

            Result::<_, ParseNumError>::Ok(pd)
        }),
    )
}

fn le_count_fct<'a, E>(count_type: CountType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context("plyers::de::body::le_count_fct", move |input| {
        trace!("Parsing LE count data as {} and casting to usize", count_type);
        match count_type {
            CountType::U8 => map(le_u8, |n| n as usize)(input),
            CountType::U16 => map(le_u16, |n| n as usize)(input),
            CountType::U32 => map(le_u32, |n| n as usize)(input),
            CountType::U64 => map(le_u64, |n| n as usize)(input),
        }
    })
}

fn le_number_fct<'a, E>(data_type: DataType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Value, E>
where
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]>,
{
    context("plyers::de::body::le_number_fct", move |input| {
        trace!("Parsing LE property data as {}", data_type);
        match data_type {
            DataType::U8 => map(le_u8, Value::from)(input),
            DataType::I8 => map(le_i8, Value::from)(input),
            DataType::U16 => map(le_u16, Value::from)(input),
            DataType::I16 => map(le_i16, Value::from)(input),
            DataType::U32 => map(le_u32, Value::from)(input),
            DataType::I32 => map(le_i32, Value::from)(input),
            DataType::U64 => map(le_u64, Value::from)(input),
            DataType::I64 => map(le_i64, Value::from)(input),
            DataType::F32 => map(le_f32, Value::from)(input),
            DataType::F64 => map(le_f64, Value::from)(input),
        }
    })
}

fn be_count_fct<'a, E>(count_type: CountType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context("plyers::de::body::be_count_fct", move |input| {
        trace!("Parsing BE count data as {} and casting to usize", count_type);
        match count_type {
            CountType::U8 => map(be_u8, |n| n as usize)(input),
            CountType::U16 => map(be_u16, |n| n as usize)(input),
            CountType::U32 => map(be_u32, |n| n as usize)(input),
            CountType::U64 => map(be_u64, |n| n as usize)(input),
        }
    })
}

fn be_number_fct<'a, E>(data_type: DataType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Value, E>
where
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]>,
{
    context("plyers::de::body::be_number_fct", move |input| {
        trace!("Parsing BE property data as {}", data_type);
        match data_type {
            DataType::U8 => map(be_u8, Value::from)(input),
            DataType::I8 => map(be_i8, Value::from)(input),
            DataType::U16 => map(be_u16, Value::from)(input),
            DataType::I16 => map(be_i16, Value::from)(input),
            DataType::U32 => map(be_u32, Value::from)(input),
            DataType::I32 => map(be_i32, Value::from)(input),
            DataType::U64 => map(be_u64, Value::from)(input),
            DataType::I64 => map(be_i64, Value::from)(input),
            DataType::F32 => map(be_f32, Value::from)(input),
            DataType::F64 => map(be_f64, Value::from)(input),
        }
    })
}

fn property_scalar_fct<'a, F, P, E>(num_fn: &'a F, dt: DataType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Value, E>
where
    F: Fn(DataType) -> P,
    P: FnMut(&'a [u8]) -> IResult<&'a [u8], Value, E>,
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context("plyers::de::body::property_scalar_fct", num_fn(dt))
}

fn property_list_fct<'a, F1, F2, P1, P2, E>(
    cnt_fn: &'a F1,
    num_fn: &'a F2,
    ct: CountType,
    dt: DataType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<Value>, E>
where
    F1: Fn(CountType) -> P1,
    F2: Fn(DataType) -> P2,
    P1: FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>,
    P2: FnMut(&'a [u8]) -> IResult<&'a [u8], Value, E>,
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context(
        "plyers::de::body::property_list_fct",
        length_count(cnt_fn(ct), num_fn(dt)),
    )
}

fn properties_fct<'a, 'b, F1, F2, P1, P2, E>(
    cnt_fn: &'a F1,
    num_fn: &'a F2,
    properties: &'b BTreeMap<PropertyId, PropertyDescriptor>,
    repetitions: usize,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], BTreeMap<PropertyId, (Primitive, Values)>, E> + 'b
where
    'a: 'b,
    F1: Fn(CountType) -> P1,
    F2: Fn(DataType) -> P2,
    P1: FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>,
    P2: FnMut(&'a [u8]) -> IResult<&'a [u8], Value, E>,
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]> + 'b,
{
    let mut p_iter = properties.iter().cycle();

    context(
        "plyers::de::body::properties_fct",
        fold_exact(
            properties.len() * repetitions,
            move |input| {
                let (&p_id, p_desc) = p_iter.next().unwrap();

                match p_desc {
                    PropertyDescriptor::Scalar { data_type, name, .. } => {
                        trace!("Parsing property {} as scalar data with type {}", name, data_type);
                        map(property_scalar_fct(num_fn, *data_type), |p| {
                            (p_id, Primitive::Single, *data_type, vec![p])
                        })(input)
                    }
                    PropertyDescriptor::List {
                        count_type,
                        data_type,
                        name,
                        ..
                    } => {
                        trace!(
                            "Parsing property {} as list data with type {} and count type {}",
                            name,
                            data_type,
                            count_type
                        );
                        map(property_list_fct(cnt_fn, num_fn, *count_type, *data_type), |ps| {
                            let prim = Primitive::from(ps.len());
                            (p_id, prim, *data_type, ps)
                        })(input)
                    }
                }
            },
            BTreeMap::<PropertyId, (Primitive, Values)>::new,
            |mut p_acc, (p_id, prim, dt, p)| {
                if let std::collections::btree_map::Entry::Vacant(e) = p_acc.entry(p_id) {
                    e.insert((prim, (dt, p).try_into().unwrap()));
                } else if let Some((prim_acc, ref mut p_acc)) = p_acc.get_mut(&p_id) {
                    if prim_acc != &prim {
                        *prim_acc = Primitive::Mixed;
                    }
                    p_acc.try_extend(p).unwrap();
                }

                p_acc
            },
        ),
    )
}

fn elements_fct<'a, 'b, F1, F2, P1, P2, E>(
    cnt_fn: &'a F1,
    num_fn: &'a F2,
    elements: &'b BTreeMap<ElementId, ElementDescriptor>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], BTreeMap<PropertyId, (Primitive, Values)>, E> + 'b
where
    'a: 'b,
    F1: Fn(CountType) -> P1,
    F2: Fn(DataType) -> P2,
    P1: FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>,
    P2: FnMut(&'a [u8]) -> IResult<&'a [u8], Value, E>,
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]> + 'b,
{
    let mut e_iter = elements.iter();

    context(
        "plyers::de::body::elements_fct#0",
        fold_exact(
            elements.len(),
            move |input: &'a [u8]| {
                let (_, e_desc) = e_iter.next().unwrap();
                trace!("Parsing data for element {}", e_desc.name);
                context(
                    "plyers::de::body::elements_fct#1",
                    properties_fct(cnt_fn, num_fn, &e_desc.properties, e_desc.count),
                )(input)
            },
            BTreeMap::<PropertyId, (Primitive, Values)>::new,
            |mut p_acc, e_values| {
                p_acc.extend(e_values);
                p_acc
            },
        ),
    )
}

pub fn body_fct<
    'a,
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]> + 'a,
>(
    ply: PlyDescriptor,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], BTreeMap<PropertyId, (Primitive, Values)>, E> {
    context("plyers::de::body::body_fct", move |input| match ply.format_type {
        FormatType::Ascii => {
            debug!("Parsing PLY data as ASCII");
            elements_fct(&ascii_count_fct, &ascii_number_fct, &ply.elements)(input)
        }
        FormatType::BinaryLittleEndian => {
            debug!("Parsing PLY data as binary little endian");
            elements_fct(&le_count_fct, &le_number_fct, &ply.elements)(input)
        }
        FormatType::BinaryBigEndian => {
            debug!("Parsing PLY data as binary big endian");
            elements_fct(&be_count_fct, &be_number_fct, &ply.elements)(input)
        }
    })
}

#[cfg(test)]
mod tests {
    use nom::number::complete::recognize_float;
    use proptest::{prop_assert_eq, proptest, string::bytes_regex};

    const EMPTY: &[u8] = b"";

    proptest! {
        #[test]
        fn recognize_float_behaves_as_expected(ref input in bytes_regex(r"[-+]?([0-9]*[.])?[0-9]+([eE][-+]?[0-9]+)?").unwrap()) {
            prop_assert_eq!(recognize_float::<_, nom::error::Error<&[u8]>>(&input[..]), Ok((EMPTY, &input[..])))
        }
    }
}
