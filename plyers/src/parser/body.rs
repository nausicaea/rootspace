use std::collections::BTreeMap;

use either::Either::{self, Left, Right};
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
use num_traits::{cast, NumCast};

use super::{
    common::{fold_exact, is_whitespace, whitespace},
    ParseNumError,
};
use crate::types::{
    CountType, DataType, ElementDescriptor, ElementId, FormatType, PlyDescriptor, PropertyDescriptor, PropertyId,
};

fn ascii_count_fct<'a, E>(_count_type: CountType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>
where
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]>,
{
    context(
        "plyers::parser::body::ascii_count_fct",
        map_res(terminated(take_till1(is_whitespace), whitespace), |cd| {
            let cd = std::str::from_utf8(cd)?;
            let cd = cd.parse::<usize>()?;
            Result::<_, ParseNumError>::Ok(cd)
        }),
    )
}

fn ascii_number_fct<'a, T: NumCast, E>(data_type: DataType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]>,
{
    context(
        "plyers::parser::body::ascii_number_fct",
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
        }),
    )
}

fn le_count_fct<'a, E>(count_type: CountType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context("plyers::parser::body::le_count_fct", move |input| match count_type {
        CountType::U8 => map(le_u8, |n| n as usize)(input),
        CountType::U16 => map(le_u16, |n| n as usize)(input),
        CountType::U32 => map(le_u32, |n| n as usize)(input),
        CountType::U64 => map(le_u64, |n| n as usize)(input),
    })
}

fn le_number_fct<'a, T: NumCast, E>(data_type: DataType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]>,
{
    context("plyers::parser::body::le_number_fct", move |input| match data_type {
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
    })
}

fn be_count_fct<'a, E>(count_type: CountType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context("plyers::parser::body::be_count_fct", move |input| match count_type {
        CountType::U8 => map(be_u8, |n| n as usize)(input),
        CountType::U16 => map(be_u16, |n| n as usize)(input),
        CountType::U32 => map(be_u32, |n| n as usize)(input),
        CountType::U64 => map(be_u64, |n| n as usize)(input),
    })
}

fn be_number_fct<'a, T: NumCast, E>(data_type: DataType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]>,
{
    context("plyers::parser::body::be_number_fct", move |input| match data_type {
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
    })
}

fn property_scalar_fct<'a, T, F, P, E>(num_fn: &'a F, dt: DataType) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    T: NumCast,
    F: Fn(DataType) -> P,
    P: FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>,
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context("plyers::parser::body::property_scalar_fct", num_fn(dt))
}

fn property_list_fct<'a, T, F1, F2, P1, P2, E>(
    cnt_fn: &'a F1,
    num_fn: &'a F2,
    ct: CountType,
    dt: DataType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<T>, E>
where
    T: NumCast,
    F1: Fn(CountType) -> P1,
    F2: Fn(DataType) -> P2,
    P1: FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>,
    P2: FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>,
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context(
        "plyers::parser::body::property_list_fct",
        length_count(cnt_fn(ct), num_fn(dt)),
    )
}

fn properties_fct<'a, 'b, V, I, F1, F2, F3, P1, P2, P3, E>(
    cnt_fn: &'a F1,
    v_num_fn: &'a F2,
    i_num_fn: &'a F3,
    properties: &'b BTreeMap<PropertyId, PropertyDescriptor>,
    repetitions: usize,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], BTreeMap<PropertyId, Either<Vec<V>, Vec<Vec<I>>>>, E> + 'b
where
    'a: 'b,
    V: NumCast + 'b,
    I: NumCast + 'b,
    F1: Fn(CountType) -> P1,
    F2: Fn(DataType) -> P2,
    F3: Fn(DataType) -> P3,
    P1: FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>,
    P2: FnMut(&'a [u8]) -> IResult<&'a [u8], V, E>,
    P3: FnMut(&'a [u8]) -> IResult<&'a [u8], I, E>,
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]> + 'b,
{
    let mut p_iter = properties.iter().cycle();

    context(
        "plyers::parser::body::properties_fct",
        fold_exact(
            properties.len() * repetitions,
            move |input| {
                let (&p_id, p_desc) = p_iter.next().unwrap();

                match p_desc {
                    PropertyDescriptor::Scalar { data_type, .. } => {
                        map(property_scalar_fct(v_num_fn, *data_type), |p| (p_id, Left(p)))(input)
                    }
                    PropertyDescriptor::List {
                        count_type, data_type, ..
                    } => map(property_list_fct(cnt_fn, i_num_fn, *count_type, *data_type), |ps| {
                        (p_id, Right(ps))
                    })(input),
                }
            },
            BTreeMap::<PropertyId, Either<Vec<V>, Vec<Vec<I>>>>::new,
            |mut p_acc, (p_id, p)| {
                if !p_acc.contains_key(&p_id) {
                    p_acc.insert(
                        p_id,
                        match p {
                            Left(ps) => Left(vec![ps]),
                            Right(pl) => Right(vec![pl]),
                        },
                    );
                } else {
                    p_acc.get_mut(&p_id).map(|p_acc_entry| match p_acc_entry {
                        Left(ref mut ps_acc) => p.map_left(|ps| ps_acc.push(ps)).left().unwrap_or_default(),
                        Right(ref mut pl_acc) => p.map_right(|pl| pl_acc.push(pl)).right().unwrap_or_default(),
                    });
                }

                p_acc
            },
        ),
    )
}

fn elements_fct<'a, 'b, V, I, F1, F2, F3, P1, P2, P3, E>(
    cnt_fn: &'a F1,
    v_num_fn: &'a F2,
    i_num_fn: &'a F3,
    elements: &'b BTreeMap<ElementId, ElementDescriptor>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], BTreeMap<ElementId, BTreeMap<PropertyId, Either<Vec<V>, Vec<Vec<I>>>>>, E> + 'b
where
    'a: 'b,
    V: NumCast + 'b,
    I: NumCast + 'b,
    F1: Fn(CountType) -> P1,
    F2: Fn(DataType) -> P2,
    F3: Fn(DataType) -> P3,
    P1: FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>,
    P2: FnMut(&'a [u8]) -> IResult<&'a [u8], V, E>,
    P3: FnMut(&'a [u8]) -> IResult<&'a [u8], I, E>,
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]> + 'b,
{
    let mut e_iter = elements.iter();

    context(
        "plyers::parser::body::elements_fct",
        fold_exact(
            elements.len(),
            move |input: &'a [u8]| {
                let (e_id, e_desc) = e_iter.next().unwrap();

                map(
                    properties_fct(cnt_fn, v_num_fn, i_num_fn, &e_desc.properties, e_desc.count),
                    |e| (*e_id, e),
                )(input)
            },
            BTreeMap::<ElementId, BTreeMap<PropertyId, Either<Vec<V>, Vec<Vec<I>>>>>::new,
            |mut e_acc, (e_id, e)| {
                e_acc.insert(e_id, e);
                e_acc
            },
        ),
    )
}

pub fn body_fct<
    'a,
    V: NumCast + 'a,
    I: NumCast + 'a,
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]> + 'a,
>(
    ply: PlyDescriptor,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], BTreeMap<ElementId, BTreeMap<PropertyId, Either<Vec<V>, Vec<Vec<I>>>>>, E>
{
    context("plyers::parser::body::body_fct", move |input| match ply.format_type {
        FormatType::Ascii => elements_fct(&ascii_count_fct, &ascii_number_fct, &ascii_number_fct, &ply.elements)(input),
        FormatType::BinaryLittleEndian => {
            elements_fct(&le_count_fct, &le_number_fct, &le_number_fct, &ply.elements)(input)
        }
        FormatType::BinaryBigEndian => {
            elements_fct(&be_count_fct, &be_number_fct, &be_number_fct, &ply.elements)(input)
        }
    })
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
