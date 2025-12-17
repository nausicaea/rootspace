use std::fmt::Debug;

use super::{
    resource::Resource, resources::Resources, system::System, with_dependencies::WithDependencies,
    with_resources::WithResources,
};

/// An element within the heterogeneous list.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Element<H, T> {
    pub head: H,
    pub tail: T,
}

impl<H, T> Element<H, T> {
    /// Create a new `Element`, given a head and a tail argument.
    pub fn new(head: H, tail: T) -> Self {
        Element { head, tail }
    }
}

impl<H, T> WithResources for Element<H, T>
where
    H: WithResources,
    T: WithResources,
{
    #[tracing::instrument(skip_all)]
    fn with_res(res: &Resources) -> anyhow::Result<Self> {
        Ok(Self {
            head: H::with_res(res)?,
            tail: T::with_res(res)?,
        })
    }
}

impl<D, H, T> WithDependencies<D> for Element<H, T>
where
    D: Debug,
    H: WithDependencies<D>,
    T: WithDependencies<D>,
{
    #[tracing::instrument(skip_all)]
    fn with_deps(deps: &D) -> anyhow::Result<Self> {
        Ok(Self {
            head: H::with_deps(deps)?,
            tail: T::with_deps(deps)?,
        })
    }
}

/// The end of the heterogeneous list;
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct End;

impl WithResources for End {
    #[tracing::instrument(skip_all)]
    fn with_res(_: &Resources) -> anyhow::Result<Self> {
        Ok(Self)
    }
}

impl<D> WithDependencies<D> for End {
    #[tracing::instrument(skip_all)]
    fn with_deps(_: &D) -> anyhow::Result<Self> {
        Ok(Self)
    }
}

#[macro_export]
macro_rules! impl_registry {
    ($name:ident, where Head: $bound:tt $(+ $others:tt)*) => {
        pub trait $name: Sized {
            /// Statically provides the length of the heterogeneous list.
            const LEN: usize;

            /// Refers to the type associated with the head element of the list.
            type Head: $bound $(+ $others)*;
            /// Refers to the type of the tail of the list.
            type Tail: $name;

            fn is_empty() -> bool {
                Self::LEN == 0
            }

            fn contains<E: 'static>(element: &E) -> bool;

            /// Push a new element onto the head of the heterogeneous list.
            fn push<E>(self, element: E) -> $crate::registry::Element<E, Self>
            where
                E: $bound $(+ $others)*,
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

            /// Destructure into head and tail
            fn unzip(self) -> (Self::Head, Self::Tail);
        }

        impl<H, T> $name for $crate::registry::Element<H, T>
        where
            H: $bound $(+ $others)*,
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

            fn unzip(self) -> (Self::Head, Self::Tail) {
                (self.head, self.tail)
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
                self
            }

            fn unzip(self) -> (Self::Head, Self::Tail) {
                ((), self)
            }
        }
    };
}

impl_registry!(ResourceRegistry, where Head: Resource);
impl_registry!(SystemRegistry, where Head: System);

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[derive(Default, Debug, PartialEq)]
    struct TestElementA(usize);

    impl Resource for TestElementA {}

    #[derive(Default, Debug, PartialEq)]
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
            Element::new(TestElementB::default(), Element::new(TestElementA::default(), End))
        );

        let l: Element<TestElementA, Element<TestElementB, End>> =
            ResourceRegistry::push(End, TestElementB::default()).push(TestElementA::default());
        assert_eq!(
            l,
            Element::new(TestElementA::default(), Element::new(TestElementB::default(), End))
        );
    }

    #[test]
    fn eval_arbitrary_recursive() {
        let h = ResourceRegistry::push(End, TestElementA::default()).push(TestElementB::default());

        fn eval<H: ResourceRegistry>(list: &H) {
            if H::LEN > 0 {
                let _head: &H::Head = list.head();
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

    proptest! {
        #[test]
        fn push_n_induction(n in 0usize..1000) {
            let list_n = End;
            for i in 0usize..n {
                let list_nm1 = list_n;
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
                let list_nm1 = list_n;
                let list_n = ResourceRegistry::push(list_n, TestElementA(i));

                if i == n - 1 {
                    prop_assert_eq!(ResourceRegistry::len(&list_n), ResourceRegistry::len(&list_nm1) + 1);
                }
            }
        }
    }
}
