use crate::resource::Resource;
use typename::TypeName;
use serde::{Serialize, Deserialize};

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
    ($ta:ty, $tb:ty) => {
        $crate::registry::Element<$ta, $tb>
    };
    ($t:ty, $tp:tt) => {
        $crate::registry::Element<$t, $tp>
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
    ($ea:expr, $eb:expr) => {
        $crate::registry::Element::new($ea, $eb)
    };
    ($e:expr, $rest:tt) => {
        $crate::registry::Element::new($e, $rest)
    };
    ($t:expr, $($rest:tt)+) => {
        $crate::registry::Element::new($t, $crate::reg_add![$($rest)+])
    };
}

/// The `Registry` is used to register types with the world, so that the ecs can manage
/// serialization and deserialization of the entire world state without knowing the specific types
/// stored within it. The `Registry` is implemented as a heterogeneous list (more or less
/// equivalent to a series of nested two-tuples).
pub trait Registry: Sized {
    /// Statically provides the length of the `Registry`.
    const LEN: usize;

    /// Signifies the type stored at the head position of the list.
    type Head: Resource + TypeName + Serialize + for<'de> Deserialize<'de>;
    /// Signifies the type of the rest of the list.
    type Tail: Registry;

    /// Push an element to the head of the heterogeneous list.
    fn push<T>(self, element: T) -> Element<T, Self>
    where
        T: Resource + TypeName + Serialize + for<'de> Deserialize<'de>,
    {
        Element::new(element, self)
    }

    /// Return the length of the heterogeneous list.
    fn len(&self) -> usize {
        Self::LEN
    }

    /// Return a reference to the current head of the heterogeneous list.
    fn head(&self) -> &Self::Head;

    /// Return a reference to the tail of the heterogeneous list.
    fn tail(&self) -> &Self::Tail;

    // /// Apply a closure to each element of the heterogeneous list.
    // fn for_each<F>(&self, f: &F)
    // where
    //     F: Fn(&Self::Head),
    // {
    //     if <Self as Registry>::LEN > 0 {
    //         let head = self.head();
    //         f(head);
    //         self.tail().for_each(f);
    //     }
    // }
}

/// An element within the `Registry`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Element<H, T>(H, T);

impl<H, T> Element<H, T>
where
    T: Registry,
{
    /// Create a new `Element`, given a head and a tail argument.
    pub fn new(head: H, tail: T) -> Self {
        Element(head, tail)
    }
}

impl<H, T> Registry for Element<H, T>
where
    H: Resource + TypeName + Serialize + for<'de> Deserialize<'de>,
    T: Registry,
{
    const LEN: usize = 1 + <T as Registry>::LEN;

    type Head = H;
    type Tail = T;

    fn head(&self) -> &H {
        &self.0
    }

    fn tail(&self) -> &T {
        &self.1
    }
}

/// The end of the `Registry`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct End;

impl Registry for End {
    const LEN: usize = 0;

    type Head = ();
    type Tail = End;

    fn head(&self) -> &() {
        &()
    }

    fn tail(&self) -> &End {
        &End
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_test::{Token, assert_tokens};
    use typename::TypeName;

    #[derive(Default, Debug, PartialEq, Serialize, Deserialize, TypeName)]
    struct TestElementA(usize);

    #[derive(Default, Debug, PartialEq, Serialize, Deserialize, TypeName)]
    struct TestElementB(String);

    #[test]
    fn macro_additive() {
        let a: Element<usize, Element<f32, End>> = Default::default();
        let b: RegAdd![usize, Element<f32, End>] = Default::default();
        let c: RegAdd![usize, Element<f32, End>] = reg_add![0usize, Element::new(0.0f32, End)];

        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn macros_empty() {
        let _: End = reg![];
        let _: Reg![] = reg![];
    }

    #[test]
    fn macros_single_element() {
        let _: Element<u8, End> = reg![0u8];
        let _: Reg![u8] = reg![0u8];
    }

    #[test]
    fn macros_two_elements() {
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
    fn macros_three_elements() {
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

    #[test]
    fn end() {
        let _: End = End;
    }

    #[test]
    fn push_one() {
        let l: Element<TestElementA, End> = End.push(TestElementA::default());
        assert_eq!(l, Element::new(TestElementA::default(), End));
    }

    #[test]
    fn push_two() {
        let l: Element<TestElementB, Element<TestElementA, End>> = End
            .push(TestElementA::default())
            .push(TestElementB::default());
        assert_eq!(l, Element::new(TestElementB::default(), Element::new(TestElementA::default(), End)));

        let l: Element<TestElementA, Element<TestElementB, End>> = End
            .push(TestElementB::default())
            .push(TestElementA::default());
        assert_eq!(l, Element::new(TestElementA::default(), Element::new(TestElementB::default(), End)));
    }

    #[test]
    fn eval_arbitrary_recursive() {
        let h = End
            .push(0usize)
            .push(String::from("Hello, World"));

        fn eval<H: Registry>(list: &H) {
            if H::LEN > 0 {
                let head = list.head();
                eprintln!("{:?}", head);
                eval(list.tail());
            }
        }

        eval(&h);
    }

    #[test]
    fn len_empty() {
        let h = End;
        assert_eq!(h.len(), 0);
    }

    #[test]
    fn len_one() {
        let h = End
            .push(TestElementA::default());

        assert_eq!(h.len(), 1);
    }

    #[test]
    fn len_two() {
        let h = End
            .push(TestElementA::default())
            .push(TestElementB::default());

        assert_eq!(h.len(), 2);
    }

    #[test]
    fn serde_empty() {
        let h = End;

        assert_tokens(&h, &[
            Token::UnitStruct { name: "End" },
        ]);
    }

    #[test]
    fn serde_one() {
        let h = End
            .push(TestElementA::default());

        assert_tokens(&h, &[
            Token::TupleStruct { name: "Element", len: 2 },
            Token::NewtypeStruct { name: "TestElementA" },
            Token::U64(0),
            Token::UnitStruct { name: "End" },
            Token::TupleStructEnd,
        ]);
    }

    #[test]
    fn serde_two() {
        let h = End
            .push(1u32)
            .push(8u8);

        assert_tokens(&h, &[
            Token::TupleStruct { name: "Element", len: 2 },
            Token::U8(8),
            Token::TupleStruct { name: "Element", len: 2 },
            Token::U32(1),
            Token::UnitStruct { name: "End" },
            Token::TupleStructEnd,
            Token::TupleStructEnd,
        ]);
    }

    proptest! {
        #[test]
        fn push_n_induction(n in 0usize..1000) {
            let list_n = End;
            for i in 0usize..n {
                let list_nm1 = list_n.clone();
                let list_n = list_n.push(i);

                if i == n - 1 {
                    prop_assert_eq!(list_n, Element::new(i, list_nm1));
                }
            }
        }

        #[test]
        fn len_n_induction(n in 0usize..1000) {
            let list_n = End;
            for i in 0usize..n {
                let list_nm1 = list_n.clone();
                let list_n = list_n.push(i);

                if i == n - 1 {
                    prop_assert_eq!(list_n.len(), list_nm1.len() + 1);
                }
            }
        }
    }
}
