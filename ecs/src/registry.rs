use crate::{maybe_default::MaybeDefault, resource::Resource, system::System, with_resources::WithResources, serialization_proxy::SerializationProxy};
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::fmt::Debug;

/// An element within the heterogeneous list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Element<H, T> {
    pub head: H,
    pub tail: T,
}

impl<H, T> Element<H, T> {
    /// Create a new `Element`, given a head and a tail argument.
    pub fn new(head: H, tail: T) -> Self {
        Element {
            head,
            tail,
        }
    }
}

/// The end of the heterogeneous list;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct End;

#[macro_export]
macro_rules! impl_registry {
    ($name:ident, where Head: $bound:tt $(+ $others:tt)*) => {
        pub trait $name: Sized {
            /// Statically provides the length of the heterogeneous list.
            const LEN: usize;

            /// Refers to the type associated with the head element of the list.
            type Head: $bound $(+ $others)* + serde::Serialize + for<'de> serde::Deserialize<'de>;
            /// Refers to the type of the tail of the list.
            type Tail: $name;

            fn is_empty() -> bool {
                Self::LEN == 0
            }

            fn contains<E: 'static>(element: &E) -> bool;

            /// Push a new element onto the head of the heterogeneous list.
            fn push<E>(self, element: E) -> $crate::registry::Element<E, Self>
            where
                E: $bound $(+ $others)* + serde::Serialize + for<'de> serde::Deserialize<'de>,
            {
                $crate::registry::Element::new(element, self)
            }

            /// Return the length of the heterogeneous list.
            fn len(&self) -> usize {
                Self::LEN
            }

            /// Return a reference to the current head of the heterogeneous list.
            fn head(&self) -> &Self::Head;

            /// Return a reference to the tail of the heterogeneous list.
            fn tail(&self) -> &Self::Tail;
        }

        impl<H, T> $name for $crate::registry::Element<H, T>
        where
            H: $bound $(+ $others)* + serde::Serialize + for<'de> serde::Deserialize<'de>,
            T: $name,
        {
            type Head = H;
            type Tail = T;

            const LEN: usize = 1 + <T as $name>::LEN;

            fn contains<E: 'static>(element: &E) -> bool {
                if std::any::TypeId::of::<Self::Head>() == std::any::TypeId::of::<E>() {
                    return true;
                }

                Self::Tail::contains(element)
            }

            fn head(&self) -> &Self::Head {
                &self.head
            }

            fn tail(&self) -> &Self::Tail {
                &self.tail
            }
        }

        impl $name for $crate::registry::End {
            type Head = ();
            type Tail = $crate::registry::End;

            const LEN: usize = 0;

            fn contains<E: 'static>(_element: &E) -> bool {
                false
            }

            fn head(&self) -> &Self::Head {
                &()
            }

            fn tail(&self) -> &Self::Tail {
                &$crate::registry::End
            }
        }
    };
}

impl_registry!(ResourceRegistry, where Head: Resource + MaybeDefault + Debug + SerializationProxy);
impl_registry!(SystemRegistry, where Head: System + WithResources + Debug);

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_test::{assert_tokens, Token};

    #[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
    struct TestElementA(usize);

    impl Resource for TestElementA {}

    #[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
    struct TestElementB(String);

    impl Resource for TestElementB {}

    #[test]
    fn end() {
        let _: End = End;
    }

    #[test]
    fn push_one() {
        let l: Element<TestElementA, End> = ResourceRegistry::push(End, TestElementA::default());
        assert_eq!(l, Element::new(TestElementA::default(), End));
    }

    #[test]
    fn push_two() {
        let l: Element<TestElementB, Element<TestElementA, End>> =
            ResourceRegistry::push(End, TestElementA::default()).push(TestElementB::default());
        assert_eq!(
            l,
            Element::new(
                TestElementB::default(),
                Element::new(TestElementA::default(), End)
            )
        );

        let l: Element<TestElementA, Element<TestElementB, End>> =
            ResourceRegistry::push(End, TestElementB::default()).push(TestElementA::default());
        assert_eq!(
            l,
            Element::new(
                TestElementA::default(),
                Element::new(TestElementB::default(), End)
            )
        );
    }

    #[test]
    fn eval_arbitrary_recursive() {
        let h = ResourceRegistry::push(End, TestElementA::default()).push(TestElementB::default());

        fn eval<H: ResourceRegistry>(list: &H) {
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
        assert_eq!(ResourceRegistry::len(&h), 0);
    }

    #[test]
    fn len_one() {
        let h = ResourceRegistry::push(End, TestElementA::default());

        assert_eq!(ResourceRegistry::len(&h), 1);
    }

    #[test]
    fn len_two() {
        let h = ResourceRegistry::push(End, TestElementA::default()).push(TestElementB::default());

        assert_eq!(ResourceRegistry::len(&h), 2);
    }

    #[test]
    fn serde_empty() {
        let h = End;

        assert_tokens(&h, &[Token::UnitStruct { name: "End" }]);
    }

    #[test]
    fn serde_one() {
        let h = ResourceRegistry::push(End, TestElementA::default());

        assert_tokens(
            &h,
            &[
                Token::TupleStruct {
                    name: "Element",
                    len: 2,
                },
                Token::NewtypeStruct {
                    name: "TestElementA",
                },
                Token::U64(0),
                Token::UnitStruct { name: "End" },
                Token::TupleStructEnd,
            ],
        );
    }

    #[test]
    fn serde_two() {
        let h = ResourceRegistry::push(End, TestElementA::default()).push(TestElementB::default());

        assert_tokens(
            &h,
            &[
                Token::TupleStruct {
                    name: "Element",
                    len: 2,
                },
                Token::NewtypeStruct {
                    name: "TestElementB",
                },
                Token::Str(""),
                Token::TupleStruct {
                    name: "Element",
                    len: 2,
                },
                Token::NewtypeStruct {
                    name: "TestElementA",
                },
                Token::U64(0),
                Token::UnitStruct { name: "End" },
                Token::TupleStructEnd,
                Token::TupleStructEnd,
            ],
        );
    }

    proptest! {
        #[test]
        fn push_n_induction(n in 0usize..1000) {
            let list_n = End;
            for i in 0usize..n {
                let list_nm1 = list_n.clone();
                let list_n = ResourceRegistry::push(list_n, TestElementA(i));

                if i == n - 1 {
                    prop_assert_eq!(list_n, Element::new(TestElementA(i), list_nm1));
                }
            }
        }

        #[test]
        fn len_n_induction(n in 0usize..1000) {
            let list_n = End;
            for i in 0usize..n {
                let list_nm1 = list_n.clone();
                let list_n = ResourceRegistry::push(list_n, TestElementA(i));

                if i == n - 1 {
                    prop_assert_eq!(ResourceRegistry::len(&list_n), ResourceRegistry::len(&list_nm1) + 1);
                }
            }
        }
    }
}
