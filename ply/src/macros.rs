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
