/// Construct the type signature of a `Registry`.
///
/// # Examples
///
/// ```
/// use ecs::Reg;
///
/// let _l: Reg![usize, f32, bool] = Default::default();
/// ```
#[macro_export]
macro_rules! Reg {
    () => {
        $crate::registry::End
    };
    (...$rest:ty) => {
        $rest
    };
    ($t:ty) => {
        $crate::Reg![$t,]
    };
    ($t:ty, $($rest:tt)*) => {
        $crate::registry::Element<$t, $crate::Reg![$($rest)*]>
    };
}

/// Use this macro to expand the type of an existing `Registry`.
/// # Examples
///
/// ```
/// use ecs::{RegAdd, Element, End};
///
/// let _l: RegAdd![usize, f32, Element<bool, End>] = Default::default();
/// ```
#[macro_export]
macro_rules! RegAdd {
    ($t:ty) => {
        $t
    };
    ($ta:ty, $tb:ty) => {
        $crate::registry::Element<$ta, $tb>
    };
    ($t:ty, $($rest:tt)+) => {
        $crate::registry::Element<$t, $crate::RegAdd![$($rest)+]>
    };
}

/// Construct an instance of a `Registry`.
///
/// # Examples
///
/// ```
/// use ecs::reg;
///
/// let _l = reg![0usize, 100.0f32, false];
/// ```
#[macro_export]
macro_rules! reg {
    () => {
        $crate::registry::End
    };
    (...$rest:expr) => {
        $rest
    };
    ($e:expr) => {
        $crate::reg![$e,]
    };
    ($e:expr, $($rest:tt)*) => {
        $crate::registry::Element::new($e, $crate::reg![$($rest)*])
    };
}

/// Use this macro to add upon an existing `Registry`.
///
/// # Examples
///
/// ```
/// use ecs::{reg_add, Element, End};
///
/// let _l = reg_add![0usize, 100.0f32, Element::new(false, End)];
/// ```
#[macro_export]
macro_rules! reg_add {
    ($e:expr) => {
        $e
    };
    ($ea:expr, $eb:expr) => {
        $crate::registry::Element::new($ea, $eb)
    };
    ($t:expr, $($rest:tt)+) => {
        $crate::registry::Element::new($t, $crate::reg_add![$($rest)+])
    };
}

#[cfg(test)]
mod tests {
    use crate::registry::{Element, End};

    #[test]
    fn single_additive() {
        let a: Element<usize, End> = Default::default();
        let b: RegAdd![Element<usize, End>] = Default::default();
        let c: RegAdd![Element<usize, End>] = reg_add![Element::new(0usize, End)];

        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn two_additive() {
        let a: Element<usize, Element<f32, End>> = Default::default();
        let b: RegAdd![usize, Element<f32, End>] = Default::default();
        let c: RegAdd![usize, Element<f32, End>] = reg_add![0usize, Element::new(0.0f32, End)];

        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn empty() {
        let _: End = reg![];
        let _: Reg![] = reg![];
    }

    #[test]
    fn single_element() {
        let _: Element<u8, End> = reg![0u8];
        let _: Reg![u8] = reg![0u8];
    }

    #[test]
    fn two_elements() {
        let _: Element<u16, Element<u8, End>> = reg![
            100u16,
            0u8,
        ];
        let _: Reg![u16, u8] = reg![
            100u16,
            0u8,
        ];

        let _: Element<u8, Element<u16, End>> = reg![
            2u8,
            1u16,
        ];
        let _: Reg![u8, u16] = reg![
            2u8,
            1u16,
        ];
    }

    #[test]
    fn three_elements() {
        let _: Element<u32, Element<u16, Element<u8, End>>> = reg![
            2u32,
            100u16,
            0u8,
        ];
        let _: Reg![u32, u16, u8] = reg![
            2u32,
            100u16,
            0u8,
        ];
    }

}
