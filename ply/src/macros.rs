/// Implements various traits for `PropertyData`.
#[macro_export]
macro_rules! impl_property_data {
    (
        $(#[$outer:meta])*
        pub enum $name:ident {
            $(
                $(#[$inner:ident $(args:tt)*])*
                $variant:ident($type:ty),
            )+
        }
    ) => {
        impl_property_data! {
            $(#[$outer])*
            (pub) enum $name {
                $(
                    $(#[$inner $($args)*])*
                    $variant($type),
                )+
            }
        }
    };
    (
        $(#[$outer:meta])*
        enum $name:ident {
            $(
                $(#[$inner:ident $(args:tt)*])*
                $variant:ident($type:ty),
            )+
        }
    ) => {
        impl_property_data! {
            $(#[$outer])*
            () enum $name {
                $(
                    $(#[$inner $($args)*])*
                    $variant($type),
                )+
            }
        }
    };
    (
        $(#[$outer:meta])*
        ($($vis:tt)*) enum $name:ident {
            $(
            $(#[$inner:ident $(args:tt)*])*
            $variant:ident($type:ty),
            )+
        }
    ) => {
        $(#[$outer])*
        $($vis)* enum $name {
            $(
            $(#[$inner $($args)*])*
            $variant($type),
            )+
        }

        $(
        impl From<$type> for $name {
            fn from(value: $type) -> Self {
                $name::$variant(value)
            }
        }
        )+
    };
}

/// Implements an ASCII-formatted scalar property.
#[macro_export]
macro_rules! impl_ascii_scalar_property {
    ($name:ident, $inner:ident, $type:ty) => {
        fn $name<'a, I>() -> impl Parser<Input = I, Output = PropertyData> + 'a
        where
            I: Stream<Item = u8, Range = u8> + 'a,
            I::Error: ParseError<I::Item, I::Range, I::Position>,
        {
            lex($inner::<I, $type>()).map(|d| d.into())
        }
    };
}

/// Implements an ASCII-formatted vector/list property.
#[macro_export]
macro_rules! impl_ascii_vector_property {
    ($name:ident, $inner:ident, $type:ty) => {
        fn $name<'a, I>(count: usize) -> impl Parser<Input = I, Output = PropertyData> + 'a
        where
            I: Stream<Item = u8, Range = u8> + 'a,
            I::Error: ParseError<I::Item, I::Range, I::Position>,
        {
            count_min_max::<Vec<_>, _>(count, count, lex($inner::<I, $type>())).map(|d| d.into())
        }
    };
}

