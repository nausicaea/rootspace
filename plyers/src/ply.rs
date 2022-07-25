use either::{Either, Left, Right};

use crate::{
    parser::{
        base::{empty::empty, engram::engram, lookahead::lookahead, take_while::take_while},
        combinator::{either::either, repeat_until::repeat_until},
        Parser,
    },
    types::{
        CommentDescriptor, CountType, DataType, ElementDescriptor, FormatType, ListPropertyDescriptor,
        ObjInfoDescriptor, PlyDescriptor, PropertyDescriptor,
    },
};

pub fn ascii_usize_parser() -> impl Parser<Item = usize> {
    take_while(|b| b != b' ').and_then(|cd| {
        String::from_utf8(cd)
            .map_err(|e| e.into())
            .and_then(|cd| cd.parse::<usize>().map_err(|e| e.into()))
    })
}

pub fn ascii_number_parser(data_type: DataType) -> impl Parser<Item = Vec<u8>> + Clone {
    take_while(|b| b != b' ' && b != b'\n').and_then(move |pd| {
        dbg!(&pd);
        let pd = String::from_utf8(pd)?;
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

        Ok(pd)
    })
}

pub fn comment() -> impl Parser<Item = CommentDescriptor> + Clone {
    engram(b"comment ")
        .chain(take_while(|b| b != b'\n'))
        .and_then(|(_, c)| {
            let c = String::from_utf8(c)?;
            Ok(CommentDescriptor(c))
        })
}

pub fn obj_info() -> impl Parser<Item = ObjInfoDescriptor> + Clone {
    engram(b"obj_info ")
        .chain(take_while(|b| b != b'\n'))
        .and_then(|(_, o)| {
            let o = String::from_utf8(o)?;
            Ok(ObjInfoDescriptor(o))
        })
}

pub fn comment_or_obj_info() -> impl Parser<Item = Vec<Either<CommentDescriptor, ObjInfoDescriptor>>> + Clone {
    empty()
        .chain(either(comment(), obj_info()))
        .map(|(_, co)| co)
        .repeated()
        .optional()
        .map(|cos| cos.unwrap_or_else(Vec::new))
}

pub fn format() -> impl Parser<Item = (Vec<Either<CommentDescriptor, ObjInfoDescriptor>>, FormatType, String)> + Clone {
    empty()
        .chain(comment_or_obj_info())
        .chain(engram(b"format "))
        .chain(take_while(|b| b != b' '))
        .and_then(|(((_, co), _), ft)| {
            let ft = FormatType::try_from_bytes(&ft)?;
            Ok((co, ft))
        })
        .chain(take_while(|b| b != b'\n'))
        .and_then(|((co, ft), fv)| {
            let fv = String::from_utf8(fv)?;
            Ok((co, ft, fv))
        })
}

pub fn property() -> impl Parser<Item = (Vec<Either<CommentDescriptor, ObjInfoDescriptor>>, PropertyDescriptor)> + Clone
{
    empty()
        .chain(comment_or_obj_info())
        .chain(engram(b"property "))
        .chain(take_while(|b| b != b' '))
        .and_then(|(((_, co), _), dt)| {
            let dt = DataType::try_from_bytes(&dt)?;
            Ok((co, dt))
        })
        .chain(take_while(|b| b != b'\n'))
        .and_then(|((co, dt), n)| {
            let n = String::from_utf8(n)?;
            Ok((co, PropertyDescriptor { data_type: dt, name: n }))
        })
}

pub fn list_property() -> impl Parser<
    Item = (
        Vec<Either<CommentDescriptor, ObjInfoDescriptor>>,
        ListPropertyDescriptor,
    ),
> + Clone {
    empty()
        .chain(comment_or_obj_info())
        .chain(engram(b"property list "))
        .chain(take_while(|b| b != b' '))
        .and_then(|(((_, co), _), ct)| {
            let ct = CountType::try_from_bytes(&ct)?;
            Ok((co, ct))
        })
        .chain(take_while(|b| b != b' '))
        .and_then(|((co, ct), dt)| {
            let dt = DataType::try_from_bytes(&dt)?;
            Ok((co, ct, dt))
        })
        .chain(take_while(|b| b != b'\n'))
        .and_then(|((co, ct, dt), n)| {
            let n = String::from_utf8(n)?;
            Ok((
                co,
                ListPropertyDescriptor {
                    count_type: ct,
                    data_type: dt,
                    name: n,
                },
            ))
        })
}

pub fn element() -> impl Parser<Item = (Vec<Either<CommentDescriptor, ObjInfoDescriptor>>, ElementDescriptor)> + Clone {
    let normal_properties = empty()
        .chain(repeat_until(property(), lookahead(b'e')))
        .map(|(_, (np, _))| np);
    let list_properties = empty()
        .chain(repeat_until(list_property(), lookahead(b'e')))
        .map(|(_, (lp, _))| lp);
    let properties = empty()
        .chain(either(normal_properties, list_properties))
        .map(|(_, p)| p);

    empty()
        .chain(comment_or_obj_info())
        .chain(engram(b"element "))
        .chain(take_while(|b| b != b' '))
        .and_then(|(((_, co), _), en)| {
            let en = String::from_utf8(en)?;
            Ok((co, en))
        })
        .chain(take_while(|b| b != b'\n'))
        .and_then(|((co, en), ec)| {
            let ec = String::from_utf8(ec)?;
            let ec = ec.parse::<usize>()?;
            Ok((co, en, ec))
        })
        .chain(properties)
        .map(|((mut eco, en, ec), head)| match head {
            Left(lhead) => {
                let mut ps = Vec::new();
                for (pco, prop) in lhead {
                    eco.extend(pco);
                    ps.push(prop);
                }
                (
                    eco,
                    ElementDescriptor {
                        name: en,
                        count: ec,
                        properties: ps,
                        list_properties: vec![],
                    },
                )
            }
            Right(rhead) => {
                let mut ps = Vec::new();
                for (pco, prop) in rhead {
                    eco.extend(pco);
                    ps.push(prop);
                }
                (
                    eco,
                    ElementDescriptor {
                        name: en,
                        count: ec,
                        properties: vec![],
                        list_properties: ps,
                    },
                )
            }
        })
}

pub fn header() -> impl Parser<Item = PlyDescriptor> {
    engram(b"ply\n")
        .chain(format())
        .map(|(_, (fco, ft, fv))| (fco, ft, fv))
        .chain(repeat_until(element(), engram(b"end_header\n")))
        .map(|((mut fco, ft, fv), (head, _))| {
            let mut es = Vec::new();
            for (eco, e) in head {
                fco.extend(eco);
                es.push(e);
            }

            let mut cos = Vec::new();
            let mut ois = Vec::new();

            for co in fco {
                match co {
                    Left(c) => cos.push(c.0),
                    Right(o) => ois.push(o.0),
                }
            }

            PlyDescriptor {
                format_type: ft,
                format_version: fv,
                elements: es,
                comments: cos,
                obj_info: ois,
            }
        })
}
