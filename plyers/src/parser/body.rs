use nom::{
    bytes::complete::take_till1,
    combinator::{cond, map, map_res},
    error::{FromExternalError, ParseError},
    multi::{fold_many_m_n, length_count},
    number::complete::{
        be_f32, be_f64, be_i16, be_i32, be_i64, be_i8, be_u16, be_u32, be_u64, be_u8, le_f32, le_f64, le_i16, le_i32,
        le_i64, le_i8, le_u16, le_u32, le_u64, le_u8, recognize_float,
    },
    sequence::{pair, terminated},
    IResult,
};
use num_traits::{cast, NumCast};

use super::{
    common::{is_whitespace, whitespace},
    ParseNumError,
};
use crate::types::{CountType, DataType, ElementDescriptor, FormatType, ListPropertyData, PlyDescriptor, PropertyData};

fn ascii_count_fct<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError>>(
    _count_type: CountType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E> {
    map_res(terminated(take_till1(is_whitespace), whitespace), |cd| {
        let cd = std::str::from_utf8(cd)?;
        let cd = cd.parse::<usize>()?;
        Result::<_, ParseNumError>::Ok(cd)
    })
}

fn ascii_number_fct<'a, T: NumCast, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError>>(
    data_type: DataType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E> {
    map_res(terminated(recognize_float, whitespace), move |pd| {
        let pd = std::str::from_utf8(pd)?;
        let pd = match data_type {
            DataType::U8 => pd
                .parse::<u8>()
                .map_err(|e| ParseNumError::ParseIntError(e))
                .and_then(|pd| cast::<_, T>(pd).ok_or(ParseNumError::NumCastError))?,
            DataType::I8 => pd
                .parse::<i8>()
                .map_err(|e| ParseNumError::ParseIntError(e))
                .and_then(|pd| cast::<_, T>(pd).ok_or(ParseNumError::NumCastError))?,
            DataType::U16 => pd
                .parse::<u16>()
                .map_err(|e| ParseNumError::ParseIntError(e))
                .and_then(|pd| cast::<_, T>(pd).ok_or(ParseNumError::NumCastError))?,
            DataType::I16 => pd
                .parse::<i16>()
                .map_err(|e| ParseNumError::ParseIntError(e))
                .and_then(|pd| cast::<_, T>(pd).ok_or(ParseNumError::NumCastError))?,
            DataType::U32 => pd
                .parse::<u32>()
                .map_err(|e| ParseNumError::ParseIntError(e))
                .and_then(|pd| cast::<_, T>(pd).ok_or(ParseNumError::NumCastError))?,
            DataType::I32 => pd
                .parse::<i32>()
                .map_err(|e| ParseNumError::ParseIntError(e))
                .and_then(|pd| cast::<_, T>(pd).ok_or(ParseNumError::NumCastError))?,
            DataType::U64 => pd
                .parse::<u64>()
                .map_err(|e| ParseNumError::ParseIntError(e))
                .and_then(|pd| cast::<_, T>(pd).ok_or(ParseNumError::NumCastError))?,
            DataType::I64 => pd
                .parse::<i64>()
                .map_err(|e| ParseNumError::ParseIntError(e))
                .and_then(|pd| cast::<_, T>(pd).ok_or(ParseNumError::NumCastError))?,
            DataType::F32 => pd
                .parse::<f32>()
                .map_err(|e| ParseNumError::ParseFloatError(e))
                .and_then(|pd| cast::<_, T>(pd).ok_or(ParseNumError::NumCastError))?,
            DataType::F64 => pd
                .parse::<f64>()
                .map_err(|e| ParseNumError::ParseFloatError(e))
                .and_then(|pd| cast::<_, T>(pd).ok_or(ParseNumError::NumCastError))?,
        };

        Result::<_, ParseNumError>::Ok(pd)
    })
}

fn le_count_fct<'a, E: ParseError<&'a [u8]>>(
    count_type: CountType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E> {
    move |input| match count_type {
        CountType::U8 => map(le_u8, |n| n as usize)(input),
        CountType::U16 => map(le_u16, |n| n as usize)(input),
        CountType::U32 => map(le_u32, |n| n as usize)(input),
        CountType::U64 => map(le_u64, |n| n as usize)(input),
    }
}

fn le_number_fct<'a, T: NumCast, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError>>(
    data_type: DataType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E> {
    move |input| match data_type {
        DataType::U8 => map_res(le_u8, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::I8 => map_res(le_i8, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::U16 => map_res(le_u16, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::I16 => map_res(le_i16, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::U32 => map_res(le_u32, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::I32 => map_res(le_i32, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::U64 => map_res(le_u64, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::I64 => map_res(le_i64, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::F32 => map_res(le_f32, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::F64 => map_res(le_f64, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
    }
}

fn be_count_fct<'a, E: ParseError<&'a [u8]>>(
    count_type: CountType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E> {
    move |input| match count_type {
        CountType::U8 => map(be_u8, |n| n as usize)(input),
        CountType::U16 => map(be_u16, |n| n as usize)(input),
        CountType::U32 => map(be_u32, |n| n as usize)(input),
        CountType::U64 => map(be_u64, |n| n as usize)(input),
    }
}

fn be_number_fct<'a, T: NumCast, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError>>(
    data_type: DataType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E> {
    move |input| match data_type {
        DataType::U8 => map_res(be_u8, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::I8 => map_res(be_i8, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::U16 => map_res(be_u16, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::I16 => map_res(be_i16, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::U32 => map_res(be_u32, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::I32 => map_res(be_i32, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::U64 => map_res(be_u64, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::I64 => map_res(be_i64, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::F32 => map_res(be_f32, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
        DataType::F64 => map_res(be_f64, |n| cast::<_, T>(n).ok_or(ParseNumError::NumCastError))(input),
    }
}

fn property_fct<'a, T, F, P, E>(
    num_fn: &'a F,
    dt: DataType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], PropertyData<T>, E>
where
    T: NumCast,
    F: Fn(DataType) -> P,
    P: FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>,
    E: ParseError<&'a [u8]>,
{
    map(num_fn(dt), |r| PropertyData(r))
}

fn list_property_fct<'a, T, F1, F2, P1, P2, E>(
    cnt_fn: &'a F1,
    num_fn: &'a F2,
    ct: CountType,
    dt: DataType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], ListPropertyData<T>, E>
where
    T: NumCast,
    F1: Fn(CountType) -> P1,
    F2: Fn(DataType) -> P2,
    P1: FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>,
    P2: FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>,
    E: ParseError<&'a [u8]>,
{
    map(length_count(cnt_fn(ct), num_fn(dt)), |r| ListPropertyData(r))
}

fn properties_fct<'a, T, F, P, E>(
    num_fn: &'a F,
    types: Vec<DataType>,
    repetitions: usize,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<PropertyData<T>>, E>
where
    T: NumCast,
    F: Fn(DataType) -> P,
    P: FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>,
    E: ParseError<&'a [u8]>,
{
    let m = types.len() * repetitions;
    let mut i = 0;

    fold_many_m_n(
        m,
        m,
        move |input| {
            let r = property_fct(num_fn, types[i % types.len()])(input);
            i += 1;
            r
        },
        Vec::new,
        |mut p_acc, p| {
            p_acc.push(p);
            p_acc
        },
    )
}

fn list_properties_fct<'a, T, F1, F2, P1, P2, E>(
    cnt_fn: &'a F1,
    num_fn: &'a F2,
    types: Vec<(CountType, DataType)>,
    repetitions: usize,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<ListPropertyData<T>>, E>
where
    T: NumCast,
    F1: Fn(CountType) -> P1,
    F2: Fn(DataType) -> P2,
    P1: FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>,
    P2: FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>,
    E: ParseError<&'a [u8]>,
{
    let m = types.len() * repetitions;
    let mut i = 0;

    fold_many_m_n(
        m,
        m,
        move |input| {
            let r = list_property_fct(cnt_fn, num_fn, types[i % types.len()].0, types[i % types.len()].1)(input);
            i += 1;
            r
        },
        Vec::new,
        |mut pl_acc, pl| {
            pl_acc.push(pl);
            pl_acc
        },
    )
}

fn elements_fct<'a, T, F1, F2, P1, P2, E>(
    cnt_fn: &'a F1,
    num_fn: &'a F2,
    elements: Vec<ElementDescriptor>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<(Vec<PropertyData<T>>, Vec<ListPropertyData<T>>)>, E>
where
    T: NumCast,
    F1: Fn(CountType) -> P1,
    F2: Fn(DataType) -> P2,
    P1: FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>,
    P2: FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>,
    E: ParseError<&'a [u8]>,
{
    let m = elements.len();
    let mut i = 0;

    fold_many_m_n(
        m,
        m,
        move |input: &'a [u8]| {
            let el = &elements[i];
            let r = pair(
                cond(
                    !el.properties.is_empty(),
                    properties_fct(num_fn, el.properties.iter().map(|p| p.data_type).collect(), el.count),
                ),
                cond(
                    !el.list_properties.is_empty(),
                    list_properties_fct(
                        cnt_fn,
                        num_fn,
                        el.list_properties.iter().map(|p| (p.count_type, p.data_type)).collect(),
                        el.count,
                    ),
                ),
            )(input);
            i += 1;
            r
        },
        Vec::new,
        |mut p_acc, (p, pl)| {
            p_acc.push((p.unwrap_or_else(Vec::new), pl.unwrap_or_else(Vec::new)));
            p_acc
        },
    )
}

pub fn body_fct<'a, T: NumCast + 'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + 'a>(
    ply: PlyDescriptor,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<(Vec<PropertyData<T>>, Vec<ListPropertyData<T>>)>, E> {
    move |input| {
        let elements = ply.elements.iter().cloned().collect();
        match ply.format_type {
            FormatType::Ascii => elements_fct(&ascii_count_fct, &ascii_number_fct, elements)(input),
            FormatType::BinaryLittleEndian => elements_fct(&le_count_fct, &le_number_fct, elements)(input),
            FormatType::BinaryBigEndian => elements_fct(&be_count_fct, &be_number_fct, elements)(input),
        }
    }
}

#[cfg(test)]
mod tests {
    use nom::number::complete::recognize_float;
    use proptest::{prop_assert_eq, proptest, string::bytes_regex};

    const EMPTY: &'static [u8] = b"";

    proptest! {
        #[test]
        fn recognize_float_behaves_as_expected(ref input in bytes_regex(r"[-+]?([0-9]*[.])?[0-9]+([eE][-+]?[0-9]+)?").unwrap()) {
            prop_assert_eq!(recognize_float::<_, nom::error::Error<&[u8]>>(&input[..]), Ok((EMPTY, &input[..])))
        }
    }
}
