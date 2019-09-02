use crate::resource::Resource;
use typename::TypeName;
use serde::{de::Deserialize, ser::Serialize};

pub trait TypeRegistry: Sized {
    const LEN: usize;

    type Head: Resource + TypeName + Serialize + for<'de> Deserialize<'de>;
    type Tail: TypeRegistry;
}

use std::fmt::Debug;

pub trait MyCustomTrait<'a>: Debug {}

impl<'a> MyCustomTrait<'a> for () {}

impl<'a> MyCustomTrait<'a> for usize {}

impl<'a> MyCustomTrait<'a> for String {}

pub trait HList: Sized {
    const LEN: usize;

    type Head: for<'a> MyCustomTrait<'a>;
    type Tail: HList;

    fn push<E>(self, element: E) -> Element<E, Self>
    where
        E: for<'a> MyCustomTrait<'a>,
    {
        Element::new(element, self)
    }

    fn head(&self) -> &Self::Head;

    fn tail(&self) -> &Self::Tail;

    fn len(&self) -> usize {
        Self::LEN
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Element<H, T>(H, T);

impl<H, T: HList> Element<H, T> {
    pub fn new(head: H, tail: T) -> Self {
        Element(head, tail)
    }
}

impl<H, T> HList for Element<H, T>
where
    H: for<'a> MyCustomTrait<'a>,
    T: HList,
{
    const LEN: usize = 1 + <T as HList>::LEN;

    type Head = H;
    type Tail = T;

    fn head(&self) -> &Self::Head {
        &self.0
    }

    fn tail(&self) -> &Self::Tail {
        &self.1
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct End;

impl HList for End {
    const LEN: usize = 0;

    type Head = ();
    type Tail = End;

    fn head(&self) -> &Self::Head {
        &()
    }

    fn tail(&self) -> &Self::Tail {
        &End
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn pop_arbitrary_loop() {
    //     let h = End
    //         .push(0usize)
    //         .push(String::from("Hello, World"));

    //     fn eval<H: HList>(l: H) {
    //         for _ in 0usize..H::LEN {
    //             let (_e, l) = l.pop()
    //         }
    //     }

    //     eval(h);
    // }

    #[test]
    fn pop_artbitrary_recursive() {
        let h = End
            .push(0usize)
            .push(String::from("Hello, World"));

        fn eval<H: HList>(list: &H) {
            if H::LEN > 0 {
                let head = list.head();
                eprintln!("{:?}", head);
                eval(list.tail());
            }
        }

        eval(&h);
        assert!(false);
    }
}
