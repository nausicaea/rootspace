use crate::resource::Resource;
use typename::TypeName;
use serde::{Serialize, Deserialize};

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


pub trait Registry: Sized {
    const LEN: usize;

    type Head: Resource + TypeName + Serialize + for<'de> Deserialize<'de>;
    type Tail: Registry;

    fn push<T>(self, element: T) -> Element<T, Self>
    where
        T: Resource + TypeName + Serialize + for<'de> Deserialize<'de>,
    {
        Element::new(element, self)
    }

    fn head(&self) -> &Self::Head;

    fn tail(&self) -> &Self::Tail;

    fn len(&self) -> usize {
        Self::LEN
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Element<H, T>(H, T);

impl<H, T> Element<H, T>
where
    T: Registry,
{
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
