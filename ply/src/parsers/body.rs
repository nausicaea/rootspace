use super::base::{ascii_floating_point, ascii_signed_integral, ascii_unsigned_integral, lex};
use combine::{
    byteorder::{ByteOrder, BE, LE},
    error::ParseError,
    parser::{
        byte::num,
        combinator::{factory, no_partial, opaque},
        item::any,
        repeat::count_min_max,
        Parser,
    },
    stream::Stream,
};
use crate::types::{Body, CountType, DataType, Element, ElementData, FormatType, Header, PropertyData};
use crate::{impl_ascii_scalar_property, impl_ascii_vector_property};

impl_ascii_scalar_property!(pai8, ascii_signed_integral, i8);
impl_ascii_scalar_property!(pau8, ascii_unsigned_integral, u8);
impl_ascii_scalar_property!(pai16, ascii_signed_integral, i16);
impl_ascii_scalar_property!(pau16, ascii_unsigned_integral, u16);
impl_ascii_scalar_property!(pai32, ascii_signed_integral, i32);
impl_ascii_scalar_property!(pau32, ascii_unsigned_integral, u32);
impl_ascii_scalar_property!(paf32, ascii_floating_point, f32);
impl_ascii_scalar_property!(paf64, ascii_floating_point, f64);
impl_ascii_vector_property!(pavi8, ascii_signed_integral, i8);
impl_ascii_vector_property!(pavu8, ascii_unsigned_integral, u8);
impl_ascii_vector_property!(pavi16, ascii_signed_integral, i16);
impl_ascii_vector_property!(pavu16, ascii_unsigned_integral, u16);
impl_ascii_vector_property!(pavi32, ascii_signed_integral, i32);
impl_ascii_vector_property!(pavu32, ascii_unsigned_integral, u32);
impl_ascii_vector_property!(pavf32, ascii_floating_point, f32);
impl_ascii_vector_property!(pavf64, ascii_floating_point, f64);

fn ascii_count<'a, I>() -> impl Parser<Input = I, Output = usize> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(ascii_unsigned_integral::<I, usize>()).expected("an ascii unsigned integral interpreted as vector length")
}

fn ascii_scalar<'a, I>(data_type: DataType) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    opaque(
        move |f: &mut dyn FnMut(&mut dyn Parser<Input = _, Output = _, PartialState = _>)| match data_type {
            DataType::Int8 => f(&mut no_partial(pai8().expected("an ascii i8 scalar"))),
            DataType::Uint8 => f(&mut no_partial(pau8().expected("an ascii u8 scalar"))),
            DataType::Int16 => f(&mut no_partial(pai16().expected("an ascii i16 scalar"))),
            DataType::Uint16 => f(&mut no_partial(pau16().expected("an ascii u16 scalar"))),
            DataType::Int32 => f(&mut no_partial(pai32().expected("an ascii i32 scalar"))),
            DataType::Uint32 => f(&mut no_partial(pau32().expected("an ascii u32 scalar"))),
            DataType::Float32 => f(&mut no_partial(paf32().expected("an ascii f32 scalar"))),
            DataType::Float64 => f(&mut no_partial(paf64().expected("an ascii f64 scalar"))),
        },
    )
}

fn ascii_vector<'a, I>(data_type: DataType) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    ascii_count().then(move |c| {
        opaque(
            move |f: &mut dyn FnMut(&mut dyn Parser<Input = _, Output = _, PartialState = _>)| match data_type {
                DataType::Int8 => f(&mut no_partial(pavi8(c).expected("an ascii vector of i8"))),
                DataType::Uint8 => f(&mut no_partial(pavu8(c).expected("an ascii vector of u8"))),
                DataType::Int16 => f(&mut no_partial(pavi16(c).expected("an ascii vector of i16"))),
                DataType::Uint16 => f(&mut no_partial(pavu16(c).expected("an ascii vector of u16"))),
                DataType::Int32 => f(&mut no_partial(pavi32(c).expected("an ascii vector of i32"))),
                DataType::Uint32 => f(&mut no_partial(pavu32(c).expected("an ascii vector of u32"))),
                DataType::Float32 => f(&mut no_partial(pavf32(c).expected("an ascii vector of f32"))),
                DataType::Float64 => f(&mut no_partial(pavf64(c).expected("an ascii vector of f64"))),
            },
        )
    })
}

fn pbvi8<'a, I, T>(count: usize) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder + 'a,
{
    count_min_max::<Vec<_>, _>(count, count, any().map(|b: u8| b as i8)).map(|b| b.into())
}

fn pbvu8<'a, I, T>(count: usize) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder + 'a,
{
    count_min_max::<Vec<_>, _>(count, count, any().map(|b: u8| b)).map(|b| b.into())
}

fn pbvi16<'a, I, T>(count: usize) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder + 'a,
{
    count_min_max::<Vec<_>, _>(count, count, num::i16::<T, _>()).map(|b| b.into())
}

fn pbvu16<'a, I, T>(count: usize) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder + 'a,
{
    count_min_max::<Vec<_>, _>(count, count, num::u16::<T, _>()).map(|b| b.into())
}

fn pbvi32<'a, I, T>(count: usize) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder + 'a,
{
    count_min_max::<Vec<_>, _>(count, count, num::i32::<T, _>()).map(|b| b.into())
}

fn pbvu32<'a, I, T>(count: usize) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder + 'a,
{
    count_min_max::<Vec<_>, _>(count, count, num::u32::<T, _>()).map(|b| b.into())
}

fn pbvf32<'a, I, T>(count: usize) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder + 'a,
{
    count_min_max::<Vec<_>, _>(count, count, num::f32::<T, _>()).map(|b| b.into())
}

fn pbvf64<'a, I, T>(count: usize) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder + 'a,
{
    count_min_max::<Vec<_>, _>(count, count, num::f64::<T, _>()).map(|b| b.into())
}

fn binary_count<'a, I, T>(count_type: CountType) -> impl Parser<Input = I, Output = usize> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder,
{
    opaque(
        move |f: &mut dyn FnMut(&mut dyn Parser<Input = _, Output = _, PartialState = _>)| match count_type {
            CountType::Uint8 => f(&mut no_partial(
                any().map(|b: u8| b as usize).expected("a vector length of u8"),
            )),
            CountType::Uint16 => f(&mut no_partial(
                num::u16::<T, _>()
                    .map(|n| n as usize)
                    .expected("a vector length of u16"),
            )),
            CountType::Uint32 => f(&mut no_partial(
                num::u32::<T, _>()
                    .map(|n| n as usize)
                    .expected("a vector length of u32"),
            )),
        },
    )
}

fn binary_scalar<'a, I, T>(data_type: DataType) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder + 'a,
{
    opaque(
        move |f: &mut dyn FnMut(&mut dyn Parser<Input = _, Output = _, PartialState = _>)| match data_type {
            DataType::Int8 => f(&mut no_partial(
                any()
                    .map(|b: u8| PropertyData::Int8(b as i8))
                    .expected("a binary i8 scalar"),
            )),
            DataType::Uint8 => f(&mut no_partial(
                any().map(|b: u8| PropertyData::Uint8(b)).expected("a binary u8 scalar"),
            )),
            DataType::Int16 => f(&mut no_partial(
                num::i16::<T, _>()
                    .map(PropertyData::Int16)
                    .expected("a binary i16 scalar"),
            )),
            DataType::Uint16 => f(&mut no_partial(
                num::u16::<T, _>()
                    .map(|n| PropertyData::Uint16(n))
                    .expected("a binary u16 scalar"),
            )),
            DataType::Int32 => f(&mut no_partial(
                num::i32::<T, _>()
                    .map(|n| PropertyData::Int32(n))
                    .expected("a binary i32 scalar"),
            )),
            DataType::Uint32 => f(&mut no_partial(
                num::u32::<T, _>()
                    .map(|n| PropertyData::Uint32(n))
                    .expected("a binary u32 scalar"),
            )),
            DataType::Float32 => f(&mut no_partial(
                num::f32::<T, _>()
                    .map(|n| PropertyData::Float32(n))
                    .expected("a binary f32 scalar"),
            )),
            DataType::Float64 => f(&mut no_partial(
                num::f64::<T, _>()
                    .map(|n| PropertyData::Float64(n))
                    .expected("a binary f64 scalar"),
            )),
        },
    )
}

fn binary_vector<'a, I, T>(
    count_type: CountType,
    data_type: DataType,
) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    T: ByteOrder + 'a,
{
    binary_count::<I, T>(count_type).then(move |c| {
        opaque(
            move |f: &mut dyn FnMut(&mut dyn Parser<Input = _, Output = _, PartialState = _>)| match data_type {
                DataType::Int8 => f(&mut no_partial(pbvi8::<I, T>(c).expected("a binary vector of i8"))),
                DataType::Uint8 => f(&mut no_partial(pbvu8::<I, T>(c).expected("a binary vector of u8"))),
                DataType::Int16 => f(&mut no_partial(pbvi16::<I, T>(c).expected("a binary vector of i16"))),
                DataType::Uint16 => f(&mut no_partial(pbvu16::<I, T>(c).expected("a binary vector of u16"))),
                DataType::Int32 => f(&mut no_partial(pbvi32::<I, T>(c).expected("a binary vector of i32"))),
                DataType::Uint32 => f(&mut no_partial(pbvu32::<I, T>(c).expected("a binary vector of u32"))),
                DataType::Float32 => f(&mut no_partial(pbvf32::<I, T>(c).expected("a binary vector of f32"))),
                DataType::Float64 => f(&mut no_partial(pbvf64::<I, T>(c).expected("a binary vector of f64"))),
            },
        )
    })
}

fn property_data<'a, I>(
    format: FormatType,
    data_type: DataType,
    count_data_type: Option<CountType>,
) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    opaque(
        move |f: &mut dyn FnMut(&mut dyn Parser<Input = _, Output = _, PartialState = _>)| match format {
            FormatType::Ascii => if count_data_type.is_some() {
                f(&mut no_partial(ascii_vector(data_type)))
            } else {
                f(&mut no_partial(ascii_scalar(data_type)))
            },
            FormatType::BinaryBigEndian => if let Some(ct) = count_data_type {
                f(&mut no_partial(binary_vector::<_, BE>(ct, data_type)))
            } else {
                f(&mut no_partial(binary_scalar::<_, BE>(data_type)))
            },
            FormatType::BinaryLittleEndian => if let Some(ct) = count_data_type {
                f(&mut no_partial(binary_vector::<_, LE>(ct, data_type)))
            } else {
                f(&mut no_partial(binary_scalar::<_, LE>(data_type)))
            },
        },
    )
}

fn inner_element_data<'a, I>(format: FormatType, element: Element) -> impl Parser<Input = I, Output = ElementData> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let property_count = element.properties.len();
    let mut parsers = element
        .properties
        .into_iter()
        .map(move |p| property_data(format, p.data_type, p.count_data_type));

    let dynamic_parser = factory(move || parsers.next().expect("Premature end of the property parsers iterator"));

    count_min_max(property_count, property_count, dynamic_parser)
        .map(|properties| ElementData { properties })
        .expected("data for a single occurrence of an element")
}

fn element_data<'a, I>(format: FormatType, element: Element) -> impl Parser<Input = I, Output = Vec<ElementData>> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let element_count = element.count;
    let dynamic_parser = factory(move || inner_element_data(format, element.clone()));

    count_min_max(element_count, element_count, dynamic_parser).expected("data for all occurrences of an element")
}

pub fn body<'a, I>(header: Header) -> impl Parser<Input = I, Output = Body> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let count = header.elements.len();
    let format = header.format.format;
    let mut parsers = header.elements.into_iter().map(move |e| element_data(format, e));

    count_min_max(count, count, factory(move || parsers.next().unwrap()))
        .map(|elements| Body { elements })
        .expected("data for the entire ply file")
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::stream::{buffered::BufferedStream, state::State, ReadStream};
    use crate::types::Property;

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
            (
                &b"3 -55 1 100 "[..],
                Some(CountType::Uint8),
                DataType::Int8,
                PropertyData::Vint8(vec![-55, 1, 100]),
            ),
            (
                &b"1 55"[..],
                Some(CountType::Uint8),
                DataType::Uint8,
                PropertyData::Vuint8(vec![55]),
            ),
            (
                &b"3 -400 6000 -1 "[..],
                Some(CountType::Uint16),
                DataType::Int16,
                PropertyData::Vint16(vec![-400, 6000, -1]),
            ),
            (
                &b"3 400 6000 1 "[..],
                Some(CountType::Uint32),
                DataType::Uint16,
                PropertyData::Vuint16(vec![400, 6000, 1]),
            ),
            (
                &b"3 -400 70000 -1 "[..],
                Some(CountType::Uint8),
                DataType::Int32,
                PropertyData::Vint32(vec![-400, 70000, -1]),
            ),
            (
                &b"3 400 70000 1 "[..],
                Some(CountType::Uint16),
                DataType::Uint32,
                PropertyData::Vuint32(vec![400, 70000, 1]),
            ),
            (
                &b"3 -1.2 5.0 0 "[..],
                Some(CountType::Uint32),
                DataType::Float32,
                PropertyData::Vfloat32(vec![-1.2, 5.0, 0.0]),
            ),
            (
                &b"3 -1.2 5.0 0 "[..],
                Some(CountType::Uint8),
                DataType::Float64,
                PropertyData::Vfloat64(vec![-1.2, 5.0, 0.0]),
            ),
        ];

        for (i, c, t, o) in expected {
            let stream = BufferedStream::new(State::new(ReadStream::new(i)), 32);
            let mut parser = property_data(FormatType::Ascii, t, c);
            let r = parser.parse(stream);
            assert!(r.is_ok());
            if let Ok(r) = r {
                assert_eq!(r.0, o);
            }
        }
    }

    #[test]
    fn property_big_endian() {
        let expected = vec![
            (&[0xf6][..], None, DataType::Int8, PropertyData::Int8(-10)),
            (&[0x0a][..], None, DataType::Uint8, PropertyData::Uint8(10)),
            (&[0xfe, 0xd4][..], None, DataType::Int16, PropertyData::Int16(-300)),
            (&[0x01, 0x2c][..], None, DataType::Uint16, PropertyData::Uint16(300)),
            (
                &[0xff, 0xfe, 0xee, 0x90][..],
                None,
                DataType::Int32,
                PropertyData::Int32(-70000),
            ),
            (
                &[0x00, 0x01, 0x11, 0x70][..],
                None,
                DataType::Uint32,
                PropertyData::Uint32(70000),
            ),
            (
                &[0xc0, 0xe3, 0xd7, 0x0a][..],
                None,
                DataType::Float32,
                PropertyData::Float32(-7.12),
            ),
            (
                &[0xc0, 0x1c, 0x7a, 0xe1, 0x47, 0xae, 0x14, 0x7b][..],
                None,
                DataType::Float64,
                PropertyData::Float64(-7.12),
            ),
            (
                &[0x03, 0xc9, 0x01, 0x64][..],
                Some(CountType::Uint8),
                DataType::Int8,
                PropertyData::Vint8(vec![-55, 1, 100]),
            ),
            (
                &[0x00, 0x01, 0x37][..],
                Some(CountType::Uint16),
                DataType::Uint8,
                PropertyData::Vuint8(vec![55]),
            ),
        ];

        for (i, c, t, o) in expected {
            let stream = BufferedStream::new(State::new(ReadStream::new(i)), 32);
            let mut parser = property_data(FormatType::BinaryBigEndian, t, c);
            let r = parser.parse(stream);
            assert!(r.is_ok());
            if let Ok(r) = r {
                assert_eq!(r.0, o);
            }
        }
    }

    #[test]
    fn property_little_endian() {
        let expected = vec![
            (&[0xf6][..], None, DataType::Int8, PropertyData::Int8(-10)),
            (&[0x0a][..], None, DataType::Uint8, PropertyData::Uint8(10)),
            (&[0xd4, 0xfe][..], None, DataType::Int16, PropertyData::Int16(-300)),
            (&[0x2c, 0x01][..], None, DataType::Uint16, PropertyData::Uint16(300)),
            (
                &[0x90, 0xee, 0xfe, 0xff][..],
                None,
                DataType::Int32,
                PropertyData::Int32(-70000),
            ),
            (
                &[0x70, 0x11, 0x01, 0x00][..],
                None,
                DataType::Uint32,
                PropertyData::Uint32(70000),
            ),
            (
                &[0x0a, 0xd7, 0xe3, 0xc0][..],
                None,
                DataType::Float32,
                PropertyData::Float32(-7.12),
            ),
            (
                &[0x7b, 0x14, 0xae, 0x47, 0xe1, 0x7a, 0x1c, 0xc0][..],
                None,
                DataType::Float64,
                PropertyData::Float64(-7.12),
            ),
            (
                &[0x03, 0xc9, 0x01, 0x64][..],
                Some(CountType::Uint8),
                DataType::Int8,
                PropertyData::Vint8(vec![-55, 1, 100]),
            ),
            (
                &[0x01, 0x00, 0x37][..],
                Some(CountType::Uint16),
                DataType::Uint8,
                PropertyData::Vuint8(vec![55]),
            ),
        ];

        for (i, c, t, o) in expected {
            let stream = BufferedStream::new(State::new(ReadStream::new(i)), 32);
            let mut parser = property_data(FormatType::BinaryLittleEndian, t, c);
            let r = parser.parse(stream);
            assert!(r.is_ok());
            if let Ok(r) = r {
                assert_eq!(r.0, o);
            }
        }
    }

    #[test]
    fn element_data_ascii() {
        let element = Element {
            name: "vertex".into(),
            count: 2,
            properties: vec![
                Property {
                    name: "x".into(),
                    count_data_type: None,
                    data_type: DataType::Float32,
                },
                Property {
                    name: "y".into(),
                    count_data_type: Some(CountType::Uint8),
                    data_type: DataType::Float32,
                },
            ],
        };

        let expected = vec![
            ElementData {
                properties: vec![
                    PropertyData::Float32(100.1),
                    PropertyData::Vfloat32(vec![1.0, 2.2, 3.0]),
                ],
            },
            ElementData {
                properties: vec![
                    PropertyData::Float32(50.0),
                    PropertyData::Vfloat32(vec![0.0, -1.0, 50.0]),
                ],
            },
        ];

        let stream = BufferedStream::new(
            State::new(ReadStream::new(&b"100.1 3 1.0 2.2 3.0\n50.0 3 0.0 -1.0 50.0"[..])),
            32,
        );
        let mut parser = element_data(FormatType::Ascii, element);
        let r = parser.parse(stream);
        assert!(r.is_ok());
        if let Ok(r) = r {
            assert_eq!(r.0, expected);
        }
    }
}
