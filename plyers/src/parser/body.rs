use nom::{
    bytes::complete::take_till1,
    combinator::{cond, map, map_res},
    error::{FromExternalError, ParseError},
    multi::{fold_many_m_n, length_count},
    number::complete::{
        be_f32, be_f64, be_i16, be_i32, be_i8, be_u16, be_u32, be_u8, le_f32, le_f64, le_i16, le_i32, le_i8, le_u16,
        le_u32, le_u8, recognize_float,
    },
    sequence::{pair, terminated},
    IResult,
};

use super::{
    common::{is_whitespace, whitespace},
    ParseNumError,
};
use crate::types::{CountType, DataType, ElementDescriptor, FormatType, PlyDescriptor};

fn ascii_count_fct<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError>>(
    _count_type: CountType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E> {
    map_res(terminated(take_till1(is_whitespace), whitespace), |cd| {
        let cd = std::str::from_utf8(cd)?;
        let cd = cd.parse::<usize>()?;
        Result::<_, ParseNumError>::Ok(cd)
    })
}

fn ascii_number_fct<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError>>(
    data_type: DataType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>, E> {
    map_res(terminated(recognize_float, whitespace), move |pd| {
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
    })
}

fn le_count_fct<'a, E: ParseError<&'a [u8]>>(
    count_type: CountType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E> {
    move |input| match count_type {
        CountType::U8 => map(le_u8, |n| n as usize)(input),
        CountType::U16 => map(le_u16, |n| n as usize)(input),
        CountType::U32 => map(le_u32, |n| n as usize)(input),
    }
}

fn le_number_fct<'a, E: ParseError<&'a [u8]>>(
    data_type: DataType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>, E> {
    move |input| match data_type {
        DataType::U8 => map(le_u8, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::I8 => map(le_i8, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::U16 => map(le_u16, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::I16 => map(le_i16, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::U32 => map(le_u32, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::I32 => map(le_i32, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::F32 => map(le_f32, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::F64 => map(le_f64, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
    }
}

fn be_count_fct<'a, E: ParseError<&'a [u8]>>(
    count_type: CountType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E> {
    move |input| match count_type {
        CountType::U8 => map(be_u8, |n| n as usize)(input),
        CountType::U16 => map(be_u16, |n| n as usize)(input),
        CountType::U32 => map(be_u32, |n| n as usize)(input),
    }
}

fn be_number_fct<'a, E: ParseError<&'a [u8]>>(
    data_type: DataType,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>, E> {
    move |input| match data_type {
        DataType::U8 => map(be_u8, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::I8 => map(be_i8, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::U16 => map(be_u16, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::I16 => map(be_i16, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::U32 => map(be_u32, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::I32 => map(be_i32, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::F32 => map(be_f32, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
        DataType::F64 => map(be_f64, |n| n.to_ne_bytes().into_iter().collect::<Vec<u8>>())(input),
    }
}

fn properties_fct<'a, F, P, E>(
    num_fn: &'a F,
    types: Vec<DataType>,
    repetitions: usize,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>
where
    P: FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>,
    F: Fn(DataType) -> P,
    E: ParseError<&'a [u8]>,
{
    let m = types.len() * repetitions;
    let mut i = 0;

    fold_many_m_n(
        m,
        m,
        move |input| {
            let r = num_fn(types[i % types.len()])(input);
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

fn list_properties_fct<'a, F1, F2, P1, P2, E>(
    cnt_fn: &'a F1,
    num_fn: &'a F2,
    types: Vec<(CountType, DataType)>,
    repetitions: usize,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>
where
    P1: FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>,
    P2: FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>,
    F1: Fn(CountType) -> P1,
    F2: Fn(DataType) -> P2,
    E: ParseError<&'a [u8]>,
{
    let m = types.len() * repetitions;
    let mut i = 0;

    fold_many_m_n(
        m,
        m,
        move |input| {
            let r = length_count(cnt_fn(types[i % types.len()].0), num_fn(types[i % types.len()].1))(input);
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

fn elements_fct<'a, F1, F2, P1, P2, E>(
    cnt_fn: &'a F1,
    num_fn: &'a F2,
    elements: Vec<ElementDescriptor>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], (Vec<u8>, Vec<u8>), E>
where
    P1: FnMut(&'a [u8]) -> IResult<&'a [u8], usize, E>,
    P2: FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<u8>, E>,
    F1: Fn(CountType) -> P1,
    F2: Fn(DataType) -> P2,
    E: ParseError<&'a [u8]>,
{
    let m = elements.len();
    let mut i = 0;

    fold_many_m_n(
        m,
        m,
        move |input: &'a [u8]| {
            let el = &elements[i];
            let r = map(
                pair(
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
                ),
                |(p, pl)| (p.unwrap_or_else(Vec::new), pl.unwrap_or_else(Vec::new)),
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

pub fn body_fct<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + 'a>(
    ply: PlyDescriptor,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], (Vec<u8>, Vec<u8>), E> {
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
