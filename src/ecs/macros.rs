/// Construct the type signature of a `ResourceRegistry`.
///
/// # Examples
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use rootspace::{Reg, ecs::resource::Resource};
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct A(usize);
///
/// impl Resource for A {}
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct B(f32);
///
/// impl Resource for B {}
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct C(u8);
///
/// impl Resource for C {}
///
/// let _l: Reg![A, B, C] = Default::default();
/// ```
#[macro_export]
macro_rules! Reg {
    () => {
        $crate::ecs::registry::End
    };
    (...$rest:ty) => {
        $rest
    };
    ($t:ty) => {
        $crate::Reg![$t,]
    };
    ($t:ty, $($rest:tt)*) => {
        $crate::ecs::registry::Element<$t, $crate::Reg![$($rest)*]>
    };
}

/// Use this macro to expand the type of an existing `Registry`.
/// # Examples
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use rootspace::{RegAdd, ecs::{registry::Element, registry::End, resource::Resource}};
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct A(usize);
///
/// impl Resource for A {}
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct B(f32);
///
/// impl Resource for B {}
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct C(u8);
///
/// impl Resource for C {}
///
/// let _l: RegAdd![usize, f32, Element<bool, End>] = Default::default();
/// ```
#[macro_export]
macro_rules! RegAdd {
    ($t:ty) => {
        $t
    };
    ($ta:ty, $tb:ty) => {
        $crate::ecs::registry::Element<$ta, $tb>
    };
    ($t:ty, $($rest:tt)+) => {
        $crate::ecs::registry::Element<$t, $crate::RegAdd![$($rest)+]>
    };
}

/// Construct an instance of a `ResourceRegistry`.
///
/// # Examples
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use rootspace::{reg, ecs::resource::{Resource}};
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct A(usize);
///
/// impl Resource for A {}
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct B(f32);
///
/// impl Resource for B {}
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct C(u8);
///
/// impl Resource for C {}
///
/// let _l = reg![A::default(), B::default(), C::default()];
/// ```
#[macro_export]
macro_rules! reg {
    () => {
        $crate::ecs::registry::End
    };
    (...$rest:expr) => {
        $rest
    };
    ($e:expr) => {
        $crate::reg![$e,]
    };
    ($e:expr, $($rest:tt)*) => {
        $crate::ecs::registry::Element::new($e, $crate::reg![$($rest)*])
    };
}

/// Use this macro to add upon an existing `ResourceRegistry`.
///
/// # Examples
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use rootspace::{reg_add, ecs::{registry::Element, registry::End, resource::Resource}};
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct A(usize);
///
/// impl Resource for A {}
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct B(f32);
///
/// impl Resource for B {}
///
/// #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// struct C(u8);
///
/// impl Resource for C {}
///
/// let _l = reg_add![A::default(), B::default(), Element::new(C::default(), End)];
/// ```
#[macro_export]
macro_rules! reg_add {
    ($e:expr) => {
        $e
    };
    ($ea:expr, $eb:expr) => {
        $crate::ecs::registry::Element::new($ea, $eb)
    };
    ($t:expr, $($rest:tt)+) => {
        $crate::ecs::registry::Element::new($t, $crate::reg_add![$($rest)+])
    };
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::ecs::{
        registry::{Element, End},
        resource::Resource,
    };

    #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
    struct A(usize);

    impl Resource for A {}

    #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
    struct B(f32);

    impl Resource for B {}

    #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
    struct C(u8);

    impl Resource for C {}

    #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
    struct D(u16);

    impl Resource for D {}

    #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
    struct E(u32);

    impl Resource for E {}

    #[test]
    fn single_additive() {
        let a: Element<A, End> = Default::default();
        let b: RegAdd![Element<A, End>] = Default::default();
        let c: RegAdd![Element<A, End>] = reg_add![Element::new(A::default(), End)];

        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn two_additive() {
        let a: Element<A, Element<B, End>> = Default::default();
        let b: RegAdd![A, Element<B, End>] = Default::default();
        let c: RegAdd![A, Element<B, End>] = reg_add![A::default(), Element::new(B::default(), End)];

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
        let _: Element<C, End> = reg![C::default()];
        let _: Reg![C] = reg![C::default()];
    }

    #[test]
    fn two_elements() {
        let _: Element<D, Element<C, End>> = reg![D::default(), C::default()];
        let _: Reg![D, C] = reg![D::default(), C::default()];

        let _: Element<C, Element<D, End>> = reg![C::default(), D::default()];
        let _: Reg![C, D] = reg![C::default(), D::default()];
    }

    #[test]
    fn three_elements() {
        let _: Element<E, Element<D, Element<C, End>>> = reg![E::default(), D::default(), C::default()];
        let _: Reg![E, D, C] = reg![E::default(), D::default(), C::default()];
    }
}
