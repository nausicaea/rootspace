#![allow(non_snake_case)]

fn intersect_many<T, C>(sets: &[&std::collections::HashSet<T>]) -> C
where
    T: std::hash::Hash + Eq + Clone,
    C: std::iter::FromIterator<T>,
{
    let shortest_set = sets.iter().min_by(|&&a, &&b| a.len().cmp(&b.len()));

    shortest_set
        .iter()
        .flat_map(|&&s| s.iter())
        .filter(|&k| sets.iter().all(|&s| s.contains(k)))
        .cloned()
        .collect()
}

macro_rules! impl_joined_iter {
    ($name:ident, #reads: &$tlt:lifetime $ty:ident $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<$tlt, $ty> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $ty: &$tlt $ty,
        }

        impl<$tlt, $ty> $name<$tlt, $ty>
        where
            $ty: $crate::storage::Storage,
        {
            pub(crate) fn new($ty: &$tlt $ty) -> Self {
                $name {
                    indices: $ty.indices().iter().cloned().collect(),
                    cursor: 0,
                    $ty,
                }
            }
        }

        impl<$tlt, $ty> ExactSizeIterator for $name<$tlt, $ty>
        where
            $ty: $crate::storage::Storage,
        {}

        impl<$tlt, $ty> std::iter::FusedIterator for $name<$tlt, $ty>
        where
            $ty: $crate::storage::Storage,
        {}

        impl<$tlt, $ty> Iterator for $name<$tlt, $ty>
        where
            $ty: $crate::storage::Storage,
        {
            type Item = &$tlt $ty::Item;

            fn next(&mut self) -> Option<Self::Item> {
                if self.cursor >= self.indices.len() {
                    return None;
                }

                let idx = self.indices[self.cursor];
                self.cursor += 1;

                unsafe {
                    Some(self.$ty.get_unchecked(idx))
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining_len = self.indices
                    .len()
                    .checked_sub(self.cursor)
                    .unwrap_or(0);

                (remaining_len, Some(remaining_len))
            }
        }
    };

    ($name:ident, #writes: &$tltm:lifetime mut $tym:ident $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<$tltm, $tym> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $tym: &$tltm mut $tym,
        }

        impl<$tltm, $tym> $name<$tltm, $tym>
        where
            $tym: $crate::storage::Storage,
        {
            pub(crate) fn new($tym: &$tltm mut $tym) -> Self {
                $name {
                    indices: $tym.indices().iter().cloned().collect(),
                    cursor: 0,
                    $tym,
                }
            }
        }

        impl<$tltm, $tym> ExactSizeIterator for $name<$tltm, $tym>
        where
            $tym: $crate::storage::Storage,
        {}

        impl<$tltm, $tym> std::iter::FusedIterator for $name<$tltm, $tym>
        where
            $tym: $crate::storage::Storage,
        {}

        impl<$tltm, $tym> Iterator for $name<$tltm, $tym>
        where
            $tym: $crate::storage::Storage,
        {
            type Item = &$tltm mut $tym::Item;

            fn next(&mut self) -> Option<Self::Item> {
                if self.cursor >= self.indices.len() {
                    return None;
                }

                let idx = self.indices[self.cursor];
                self.cursor += 1;

                unsafe {
                    let $tym = self.$tym.get_unchecked_mut(idx);

                    Some(&mut *($tym as *mut _))
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining_len = self.indices
                    .len()
                    .checked_sub(self.cursor)
                    .unwrap_or(0);

                (remaining_len, Some(remaining_len))
            }
        }
    };

    ($name:ident, #reads: $(&$tlt:lifetime $ty:ident),* $(,)?) => {
        impl_joined_iter!($name, #reads: $(&$tlt $ty),*, #writes: );
    };

    ($name:ident, #writes: $(&$tltm:lifetime mut $tym:ident),* $(,)?) => {
        impl_joined_iter!($name, #reads: , #writes: $(&$tltm mut $tym),*);
    };

    ($name:ident, #reads: $(&$tlt:lifetime $ty:ident),*, #writes: $(&$tltm:lifetime mut $tym:ident),* $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $(
                $ty: &$tlt $ty,
            )*
            $(
                $tym: &$tltm mut $tym,
            )*
        }

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {
            pub(crate) fn new($($ty: &$tlt $ty,)* $($tym: &$tltm mut $tym,)*) -> Self {
                $name {
                    indices: intersect_many(&[$($ty.indices(),)* $($tym.indices(),)*]),
                    cursor: 0,
                    $(
                        $ty,
                    )*
                    $(
                        $tym,
                    )*
                }
            }
        }

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> ExactSizeIterator for $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {}

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> std::iter::FusedIterator for $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {}

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> Iterator for $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {
            type Item = ($(&$tlt $ty::Item,)* $(&$tltm mut $tym::Item,)*);

            fn next(&mut self) -> Option<Self::Item> {
                if self.cursor >= self.indices.len() {
                    return None;
                }

                let idx = self.indices[self.cursor];
                self.cursor += 1;

                unsafe {
                    $(
                        let $ty = self.$ty.get_unchecked(idx);
                    )*
                    $(
                        let $tym = self.$tym.get_unchecked_mut(idx);
                    )*

                    Some(($($ty,)* $(&mut *($tym as *mut _),)*))
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining_len = self.indices
                    .len()
                    .checked_sub(self.cursor)
                    .unwrap_or(0);

                (remaining_len, Some(remaining_len))
            }
        }
    };
}

macro_rules! impl_joined_iter_ref {
    ($name:ident, #reads: Ref<$tlt:lifetime, $ty:ident> $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<$tlt, $ty> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $ty: std::cell::Ref<$tlt, $ty>,
        }

        impl<$tlt, $ty> $name<$tlt, $ty>
        where
            $ty: $crate::storage::Storage,
        {
            pub(crate) fn new($ty: std::cell::Ref<$tlt, $ty>) -> Self {
                $name {
                    indices: $ty.indices().iter().cloned().collect(),
                    cursor: 0,
                    $ty,
                }
            }
        }

        impl<$tlt, $ty> ExactSizeIterator for $name<$tlt, $ty>
        where
            $ty: $crate::storage::Storage,
        {}

        impl<$tlt, $ty> std::iter::FusedIterator for $name<$tlt, $ty>
        where
            $ty: $crate::storage::Storage,
        {}

        impl<$tlt, $ty> Iterator for $name<$tlt, $ty>
        where
            $ty: $crate::storage::Storage,
        {
            type Item = &$tlt $ty::Item;

            fn next(&mut self) -> Option<Self::Item> {
                if self.cursor >= self.indices.len() {
                    return None;
                }

                let idx = self.indices[self.cursor];
                self.cursor += 1;

                unsafe {
                    let $ty = self.$ty.get_unchecked(idx);

                    Some(& *($ty as *const _))
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining_len = self.indices
                    .len()
                    .checked_sub(self.cursor)
                    .unwrap_or(0);

                (remaining_len, Some(remaining_len))
            }
        }
    };

    ($name:ident, #writes: RefMut<$tltm:lifetime, $tym:ident> $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<$tltm, $tym> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $tym: std::cell::RefMut<$tltm, $tym>,
        }

        impl<$tltm, $tym> $name<$tltm, $tym>
        where
            $tym: $crate::storage::Storage,
        {
            pub(crate) fn new($tym: std::cell::RefMut<$tltm, $tym>) -> Self {
                $name {
                    indices: $tym.indices().iter().cloned().collect(),
                    cursor: 0,
                    $tym,
                }
            }
        }

        impl<$tltm, $tym> ExactSizeIterator for $name<$tltm, $tym>
        where
            $tym: $crate::storage::Storage,
        {}

        impl<$tltm, $tym> std::iter::FusedIterator for $name<$tltm, $tym>
        where
            $tym: $crate::storage::Storage,
        {}

        impl<$tltm, $tym> Iterator for $name<$tltm, $tym>
        where
            $tym: $crate::storage::Storage,
        {
            type Item = &$tltm mut $tym::Item;

            fn next(&mut self) -> Option<Self::Item> {
                if self.cursor >= self.indices.len() {
                    return None;
                }

                let idx = self.indices[self.cursor];
                self.cursor += 1;

                unsafe {
                    let $tym = self.$tym.get_unchecked_mut(idx);

                    Some(&mut *($tym as *mut _))
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining_len = self.indices
                    .len()
                    .checked_sub(self.cursor)
                    .unwrap_or(0);

                (remaining_len, Some(remaining_len))
            }
        }
    };

    ($name:ident, #reads: $(Ref<$tlt:lifetime, $ty:ident>),* $(,)?) => {
        impl_joined_iter_ref!($name, #reads: $(Ref<$tlt, $ty>),*, #writes: );
    };

    ($name:ident, #writes: $(RefMut<$tltm:lifetime, $tym:ident>),* $(,)?) => {
        impl_joined_iter_ref!($name, #reads: , #writes: $(RefMut<$tltm, $tym>),*);
    };

    ($name:ident, #reads: $(Ref<$tlt:lifetime, $ty:ident>),*, #writes: $(RefMut<$tltm:lifetime, $tym:ident>),* $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $(
                $ty: std::cell::Ref<$tlt, $ty>,
            )*
            $(
                $tym: std::cell::RefMut<$tltm, $tym>,
            )*
        }

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {
            pub(crate) fn new($($ty: std::cell::Ref<$tlt, $ty>,)* $($tym: std::cell::RefMut<$tltm, $tym>,)*) -> Self {
                $name {
                    indices: intersect_many(&[$($ty.indices(),)* $($tym.indices(),)*]),
                    cursor: 0,
                    $(
                        $ty,
                    )*
                    $(
                        $tym,
                    )*
                }
            }
        }

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> ExactSizeIterator for $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {}

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> std::iter::FusedIterator for $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {}

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> Iterator for $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {
            type Item = ($(&$tlt $ty::Item,)* $(&$tltm mut $tym::Item,)*);

            fn next(&mut self) -> Option<Self::Item> {
                if self.cursor >= self.indices.len() {
                    return None;
                }

                let idx = self.indices[self.cursor];
                self.cursor += 1;

                unsafe {
                    $(
                        let $ty = self.$ty.get_unchecked(idx);
                    )*
                    $(
                        let $tym = self.$tym.get_unchecked_mut(idx);
                    )*

                    Some(($(& *($ty as *const _),)* $(&mut *($tym as *mut _),)*))
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining_len = self.indices
                    .len()
                    .checked_sub(self.cursor)
                    .unwrap_or(0);

                (remaining_len, Some(remaining_len))
            }
        }
    };
}

impl_joined_iter!(RIter, #reads: &'a A);
impl_joined_iter!(WIter, #writes: &'a mut A);

impl_joined_iter!(RRIter, #reads: &'a A, &'b B);
impl_joined_iter!(RWIter, #reads: &'a A, #writes: &'b mut B);
impl_joined_iter!(WWIter, #writes: &'a mut A, &'b mut B);

impl_joined_iter_ref!(RIterRef, #reads: Ref<'a, A>);
impl_joined_iter_ref!(WIterRef, #writes: RefMut<'a, A>);

impl_joined_iter_ref!(RRIterRef, #reads: Ref<'a, A>, Ref<'b, B>);
impl_joined_iter_ref!(RWIterRef, #reads: Ref<'a, A>, #writes: RefMut<'b, B>);
impl_joined_iter_ref!(WWIterRef, #writes: RefMut<'a, A>, RefMut<'b, B>);

impl_joined_iter_ref!(RRRIterRef, #reads: Ref<'a, A>, Ref<'b, B>, Ref<'c, C>);
impl_joined_iter_ref!(RRWIterRef, #reads: Ref<'a, A>, Ref<'b, B>, #writes: RefMut<'c, C>);
impl_joined_iter_ref!(RWWIterRef, #reads: Ref<'a, A>, #writes: RefMut<'b, B>, RefMut<'c, C>);
impl_joined_iter_ref!(WWWIterRef, #writes: RefMut<'a, A>, RefMut<'b, B>, RefMut<'c, C>);

pub struct EnumRIter<'a, T> {
    indices: Vec<crate::entity::index::Index>,
    cursor: usize,
    data: &'a T,
}

impl<'a, T> EnumRIter<'a, T>
where
    T: crate::storage::Storage,
{
    pub(crate) fn new(data: &'a T) -> Self {
        EnumRIter {
            indices: data.indices().iter().cloned().collect(),
            cursor: 0,
            data,
        }
    }
}

impl<'a, T> ExactSizeIterator for EnumRIter<'a, T> where T: crate::storage::Storage {}

impl<'a, T> std::iter::FusedIterator for EnumRIter<'a, T> where T: crate::storage::Storage {}

impl<'a, T> Iterator for EnumRIter<'a, T>
where
    T: crate::storage::Storage,
{
    type Item = (crate::entity::index::Index, &'a T::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.indices.len() {
            return None;
        }

        let idx = self.indices[self.cursor];
        self.cursor += 1;

        unsafe { Some((idx, self.data.get_unchecked(idx))) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_len = self.indices.len().saturating_sub(self.cursor);

        (remaining_len, Some(remaining_len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{vec_storage::VecStorage, Storage};
    use std::cell::Ref;

    #[test]
    fn r_iter() {
        let mut a: VecStorage<usize> = VecStorage::default();
        a.insert(0usize, 100usize);
        a.insert(1usize, 101usize);
        a.insert(2usize, 102usize);

        let mut riter = RIter::new(&a);
        assert_eq!(riter.next(), Some(&100usize));
        assert_eq!(riter.next(), Some(&101usize));
        assert_eq!(riter.next(), Some(&102usize));
        assert_eq!(riter.next(), None);
    }

    #[test]
    fn w_iter() {
        let mut a: VecStorage<usize> = VecStorage::default();
        a.insert(0usize, 100usize);
        a.insert(1usize, 101usize);
        a.insert(2usize, 102usize);

        let mut witer = WIter::new(&mut a);
        assert_eq!(witer.next(), Some(&mut 100usize));
        assert_eq!(witer.next(), Some(&mut 101usize));
        assert_eq!(witer.next(), Some(&mut 102usize));
        assert_eq!(witer.next(), None);
    }

    #[test]
    fn rr_iter() {
        let mut a: VecStorage<usize> = VecStorage::default();
        a.insert(0usize, 100usize);
        a.insert(1usize, 101usize);
        a.insert(2usize, 102usize);

        let mut b: VecStorage<usize> = VecStorage::default();
        b.insert(0usize, 100usize);
        b.insert(1usize, 101usize);
        b.insert(2usize, 102usize);

        for (ca, cb) in RRIter::new(&a, &b) {
            std::convert::identity(ca);
            std::convert::identity(cb);
        }
    }
}
