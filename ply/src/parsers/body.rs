use super::base::{lex, ascii_unsigned_integral, ascii_signed_integral, ascii_floating_point};
use types::{Body, DataType, CountType, Element, ElementData, FormatType, Header, Property, PropertyData};
use combine::{
    byteorder::ByteOrder,
    error::ParseError,
    parser::{
        byte::num,
        combinator::{factory, no_partial, FnOpaque},
        range::take,
        repeat::count_min_max,
        Parser,
    },
    stream::RangeStream,
};

macro_rules! impl_ascii_property {
    ($name:ident, $inner:ident, $type:ty) => {
        fn $name<'a, I>() -> FnOpaque<I, PropertyData>
        where
            I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
            I::Error: ParseError<I::Item, I::Range, I::Position>,
        {
            opaque!(no_partial(lex($inner::<I, $t>()).map(|d| d.into())))
        }
    };
}

macro_rules! impl_ascii_signed {
    ($name:ident, $t:ty) => {
        fn $name<'a, I>() -> FnOpaque<I, PropertyData>
        where
            I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
            I::Error: ParseError<I::Item, I::Range, I::Position>,
        {
            opaque!(no_partial(lex(ascii_signed_integral::<I, $t>()).map(|d| d.into())))
        }
    };
}

macro_rules! impl_ascii_unsigned {
    ($name:ident, $t:ty) => {
        fn $name<'a, I>() -> FnOpaque<I, PropertyData>
        where
            I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
            I::Error: ParseError<I::Item, I::Range, I::Position>,
        {
            opaque!(no_partial(lex(ascii_unsigned_integral::<I, $t>()).map(|d| d.into())))
        }
    };
}

macro_rules! impl_ascii_float {
    ($name:ident, $t:ty) => {
        fn $name<'a, I>() -> FnOpaque<I, PropertyData>
        where
            I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
            I::Error: ParseError<I::Item, I::Range, I::Position>,
        {
            opaque!(no_partial(lex(ascii_floating_point::<I, $t>()).map(|d| d.into())))
        }
    };
}

macro_rules! impl_ascii_vector_signed {
    ($name:ident, $t:ty) => {
        fn $name<'a, I>(count: usize) -> FnOpaque<I, PropertyData>
        where
            I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
            I::Error: ParseError<I::Item, I::Range, I::Position>,
        {
            opaque!(no_partial(count_min_max::<Vec<$t>, _>(count, count, lex(ascii_signed_integral::<I, $t>())).map(|v| v.into())))
        }
    };
}

macro_rules! impl_ascii_vector_unsigned {
    ($name:ident, $t:ty) => {
        fn $name<'a, I>(count: usize) -> FnOpaque<I, PropertyData>
        where
            I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
            I::Error: ParseError<I::Item, I::Range, I::Position>,
        {
            opaque!(no_partial(count_min_max::<Vec<$t>, _>(count, count, lex(ascii_unsigned_integral::<I, $t>())).map(|v| v.into())))
        }
    };
}

macro_rules! impl_ascii_vector_float {
    ($name:ident, $t:ty) => {
        fn $name<'a, I>(count: usize) -> FnOpaque<I, PropertyData>
        where
            I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
            I::Error: ParseError<I::Item, I::Range, I::Position>,
        {
            opaque!(no_partial(count_min_max::<Vec<$t>, _>(count, count, lex(ascii_floating_point::<I, $t>())).map(|v| v.into())))
        }
    };
}

impl_ascii_property!(pai8, ascii_signed_integral, i8);
impl_ascii_signed!(pai16, i16);
impl_ascii_signed!(pai32, i32);
impl_ascii_unsigned!(pau8, u8);
impl_ascii_unsigned!(pau16, u16);
impl_ascii_unsigned!(pau32, u32);
impl_ascii_float!(paf32, f32);
impl_ascii_float!(paf64, f64);
impl_ascii_vector_signed!(pavi8, i8);
impl_ascii_vector_signed!(pavi16, i16);
impl_ascii_vector_signed!(pavi32, i32);
impl_ascii_vector_unsigned!(pavu8, u8);
impl_ascii_vector_unsigned!(pavu16, u16);
impl_ascii_vector_unsigned!(pavu32, u32);
impl_ascii_vector_float!(pavf32, f32);
impl_ascii_vector_float!(pavf64, f64);

fn ascii_count<'a, I>() -> impl Parser<Input = I, Output = usize> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(ascii_unsigned_integral::<I, usize>())
}

fn ascii_scalar<'a, I>(data_type: DataType) -> FnOpaque<I, PropertyData>
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    match data_type {
        DataType::Int8 => pai8(),
        DataType::Uint8 => pau8(),
        DataType::Int16 => pai16(),
        DataType::Uint16 => pau16(),
        DataType::Int32 => pai32(),
        DataType::Uint32 => pau32(),
        DataType::Float32 => paf32(),
        DataType::Float64 => paf64(),
    }
}

fn ascii_vector<'a, I>(data_type: DataType) -> FnOpaque<I, PropertyData>
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    opaque!(ascii_count()
        .then(|c| match data_type {
            DataType::Int8 => pavi8(c),
            DataType::Uint8 => pavu8(c),
            DataType::Int16 => pavi16(c),
            DataType::Uint16 => pavu16(c),
            DataType::Int32 => pavi32(c),
            DataType::Uint32 => pavu32(c),
            DataType::Float32 => pavf32(c),
            DataType::Float64 => pavf64(c),
        })
    )
}

fn binary_count<'a, I, T>(count_type: CountType) -> FnOpaque<I, usize>
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder,
{
    match count_type {
        CountType::Uint8 => opaque!(take(1).map(|b: &[u8]| b[0] as usize)),
        CountType::Uint16 => opaque!(num::u16::<T, _>().map(|n| n as usize)),
        CountType::Uint32 => opaque!(num::u32::<T, _>().map(|n| n as usize)),
    }
}

fn binary_scalar<'a, I, T>(data_type: DataType) -> FnOpaque<I, PropertyData>
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder,
{
    match data_type {
        DataType::Int8 => opaque!(take(1).map(|b: &[u8]| PropertyData::Int8(b[0] as i8))),
        DataType::Uint8 => opaque!(take(1).map(|b: &[u8]| PropertyData::Uint8(b[0]))),
        DataType::Int16 => opaque!(num::i16::<T, _>().map(|n| PropertyData::Int16(n))),
        DataType::Uint16 => opaque!(num::u16::<T, _>().map(|n| PropertyData::Uint16(n))),
        DataType::Int32 => opaque!(num::i32::<T, _>().map(|n| PropertyData::Int32(n))),
        DataType::Uint32 => opaque!(num::u32::<T, _>().map(|n| PropertyData::Uint32(n))),
        DataType::Float32 => opaque!(num::f32::<T, _>().map(|n| PropertyData::Float32(n))),
        DataType::Float64 => opaque!(num::f64::<T, _>().map(|n| PropertyData::Float64(n))),
    }
}

fn binary_vector<'a, I, T>(count_type: CountType, data_type: DataType) -> FnOpaque<I, PropertyData>
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder,
{
    opaque!(binary_count::<_, T>(count_type)
        .then(|c| match data_type {
            DataType::Int8 => pavi8(c),
            DataType::Uint8 => pavu8(c),
            DataType::Int16 => pavi16(c),
            DataType::Uint16 => pavu16(c),
            DataType::Int32 => pavi32(c),
            DataType::Uint32 => pavu32(c),
            DataType::Float32 => pavf32(c),
            DataType::Float64 => pavf64(c),
        })
    )
}

fn property_data<'a, I>(
    format: FormatType,
    data_type: DataType,
    count_data_type: Option<CountType>,
) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    match format {
        FormatType::Ascii => {
            if count_data_type.is_some() {
                ascii_vector(data_type)
            } else {
                ascii_scalar(data_type)
            }
        }
        FormatType::BinaryBigEndian => {
            if let Some(ct) = count_data_type {
                big_endian_vector(ct, data_type)
            } else {
                big_endian_scalar(data_type)
            }
        },
        FormatType::BinaryLittleEndian => {
            if let Some(ct) = count_data_type {
                little_endian_vector(ct, data_type)
            } else {
                little_endian_scalar(data_type)
            }
        },
    }
}

fn element_data<'a, I>(format: FormatType, element: Element) -> impl Parser<Input = I, Output = ElementData> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let count = element.properties.len();
    let mut parsers = element.properties.into_iter().map(move |p| property_data(format, p.data_type, p.count_data_type));

    count_min_max(count, count, factory(move || parsers.next().unwrap())).map(|properties| ElementData { properties })
}

pub fn body<'a, I>(header: Header) -> impl Parser<Input = I, Output = Body> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let count = header.elements.len();
    let format = header.format.format;
    let mut parsers = header.elements.into_iter().map(move |e| element_data(format, e));

    count_min_max(count, count, factory(move || parsers.next().unwrap())).map(|elements| Body { elements })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_ascii() {
        let expected = vec![
            (&b"-10 "[..], None, DataType::Int8, PropertyData::Int8(-10)),
            (&b"10 "[..], None, DataType::Uint8, PropertyData::Uint8(10)),
            (&b"-300 "[..], None, DataType::Int16, PropertyData::Int16(-300)),
            (&b"300 "[..], None, DataType::Uint16, PropertyData::Uint16(300)),
            (&b"-70000"[..], None, DataType::Int32, PropertyData::Int32(-70000)),
            (&b"70000"[..], None, DataType::Uint32, PropertyData::Uint32(70000)),
            (&b"-7.12"[..], None, DataType::Float32, PropertyData::Float32(-7.12)),
            (&b"-7.12"[..], None, DataType::Float64, PropertyData::Float64(-7.12)),
            (&b"3 -55 1 100 "[..], Some(CountType::Uint8), DataType::Int8, PropertyData::Vint8(vec![-55, 1, 100])),
            (&b"1 55"[..], Some(CountType::Uint8), DataType::Uint8, PropertyData::Vuint8(vec![55])),
            (&b"3 -400 6000 -1 "[..], Some(CountType::Uint16), DataType::Int16, PropertyData::Vint16(vec![-400, 6000, -1])),
            (&b"3 400 6000 1 "[..], Some(CountType::Uint32), DataType::Uint16, PropertyData::Vuint16(vec![400, 6000, 1])),
            (&b"3 -400 70000 -1 "[..], Some(CountType::Uint8), DataType::Int32, PropertyData::Vint32(vec![-400, 70000, -1])),
            (&b"3 400 70000 1 "[..], Some(CountType::Uint16), DataType::Uint32, PropertyData::Vuint32(vec![400, 70000, 1])),
            (&b"3 -1.2 5.0 0 "[..], Some(CountType::Uint32), DataType::Float32, PropertyData::Vfloat32(vec![-1.2, 5.0, 0.0])),
            (&b"3 -1.2 5.0 0 "[..], Some(CountType::Uint8), DataType::Float64, PropertyData::Vfloat64(vec![-1.2, 5.0, 0.0])),
        ];

        for (i, c, t, o) in expected {
            let mut parser = property_data(FormatType::Ascii, t, c);
            assert_eq!(parser.easy_parse(i), Ok((o, &b""[..])));
        }
    }

    #[test]
    fn property_big_endian() {
        let expected = vec![
            (&[0xf6][..], None, DataType::Int8, PropertyData::Int8(-10)),
            (&[0x0a][..], None, DataType::Uint8, PropertyData::Uint8(10)),
            (&[0xfe, 0xd4][..], None, DataType::Int16, PropertyData::Int16(-300)),
            (&[0x01, 0x2c][..], None, DataType::Uint16, PropertyData::Uint16(300)),
            (&[0xff, 0xfe, 0xee, 0x90][..], None, DataType::Int32, PropertyData::Int32(-70000)),
            (&[0x00, 0x01, 0x11, 0x70][..], None, DataType::Uint32, PropertyData::Uint32(70000)),
            (&[0xc0, 0xe3, 0xd7, 0x0a][..], None, DataType::Float32, PropertyData::Float32(-7.12)),
            (&[0xc0, 0x1c, 0x7a, 0xe1, 0x47, 0xae, 0x14, 0x7b][..], None, DataType::Float64, PropertyData::Float64(-7.12)),
            (&[0x03, 0xc9, 0x01, 0x64][..], Some(CountType::Uint8), DataType::Int8, PropertyData::Vint8(vec![-55, 1, 100])),
            (&[0x00, 0x01, 0x37][..], Some(CountType::Uint16), DataType::Uint8, PropertyData::Vuint8(vec![55])),
        ];

        for (i, c, t, o) in expected {
            let mut parser = property_data(FormatType::BinaryBigEndian, t, c);
            assert_eq!(parser.easy_parse(i), Ok((o, &b""[..])));
        }
    }
}
