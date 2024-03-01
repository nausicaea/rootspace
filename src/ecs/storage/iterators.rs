#![allow(non_snake_case)]

fn intersect_many<T, C>(sets: &[&std::collections::BTreeSet<T>]) -> C
where
    T: std::hash::Hash + Eq + Ord + Clone,
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
            indices: Vec<$crate::ecs::entity::index::Index>,
            cursor: usize,
            $ty: &$tlt $ty,
        }

        impl<$tlt, $ty> $name<$tlt, $ty>
        where
            $ty: $crate::ecs::storage::Storage,
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
            $ty: $crate::ecs::storage::Storage,
        {}

        impl<$tlt, $ty> std::iter::FusedIterator for $name<$tlt, $ty>
        where
            $ty: $crate::ecs::storage::Storage,
        {}

        impl<$tlt, $ty> Iterator for $name<$tlt, $ty>
        where
            $ty: $crate::ecs::storage::Storage,
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
                    .saturating_sub(self.cursor);

                (remaining_len, Some(remaining_len))
            }
        }
    };

    ($name:ident, #writes: &$tltm:lifetime mut $tym:ident $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<$tltm, $tym> {
            indices: Vec<$crate::ecs::entity::index::Index>,
            cursor: usize,
            $tym: &$tltm mut $tym,
        }

        impl<$tltm, $tym> $name<$tltm, $tym>
        where
            $tym: $crate::ecs::storage::Storage,
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
            $tym: $crate::ecs::storage::Storage,
        {}

        impl<$tltm, $tym> std::iter::FusedIterator for $name<$tltm, $tym>
        where
            $tym: $crate::ecs::storage::Storage,
        {}

        impl<$tltm, $tym> Iterator for $name<$tltm, $tym>
        where
            $tym: $crate::ecs::storage::Storage,
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
                    .saturating_sub(self.cursor);

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
            indices: Vec<$crate::ecs::entity::index::Index>,
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
                $ty: $crate::ecs::storage::Storage,
            )*
            $(
                $tym: $crate::ecs::storage::Storage,
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
                $ty: $crate::ecs::storage::Storage,
            )*
            $(
                $tym: $crate::ecs::storage::Storage,
            )*
        {}

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> std::iter::FusedIterator for $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::ecs::storage::Storage,
            )*
            $(
                $tym: $crate::ecs::storage::Storage,
            )*
        {}

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> Iterator for $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::ecs::storage::Storage,
            )*
            $(
                $tym: $crate::ecs::storage::Storage,
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
                    .saturating_sub(self.cursor);

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
            indices: Vec<$crate::ecs::entity::index::Index>,
            cursor: usize,
            $ty: parking_lot::MappedRwLockReadGuard<$tlt, $ty>,
        }

        impl<$tlt, $ty> $name<$tlt, $ty>
        where
            $ty: $crate::ecs::storage::Storage,
        {
            pub(crate) fn new($ty: parking_lot::MappedRwLockReadGuard<$tlt, $ty>) -> Self {
                $name {
                    indices: $ty.indices().iter().cloned().collect(),
                    cursor: 0,
                    $ty,
                }
            }

            pub fn get(&mut self, index: $crate::ecs::entity::index::Index) -> Option<&$tlt $ty::Item> {
                let $ty = self.$ty.get(index)?;

                unsafe { Some(& *($ty as *const _)) }
            }
        }

        impl<$tlt, $ty> ExactSizeIterator for $name<$tlt, $ty>
        where
            $ty: $crate::ecs::storage::Storage,
        {}

        impl<$tlt, $ty> std::iter::FusedIterator for $name<$tlt, $ty>
        where
            $ty: $crate::ecs::storage::Storage,
        {}

        impl<$tlt, $ty> Iterator for $name<$tlt, $ty>
        where
            $ty: $crate::ecs::storage::Storage,
        {
            type Item = ($crate::ecs::entity::index::Index, &$tlt $ty::Item);

            fn next(&mut self) -> Option<Self::Item> {
                if self.cursor >= self.indices.len() {
                    return None;
                }

                let idx = self.indices[self.cursor];
                self.cursor += 1;

                unsafe {
                    let $ty = self.$ty.get_unchecked(idx);

                    Some((idx, & *($ty as *const _)))
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining_len = self.indices
                    .len()
                    .saturating_sub(self.cursor);

                (remaining_len, Some(remaining_len))
            }
        }
    };

    ($name:ident, #writes: RefMut<$tltm:lifetime, $tym:ident> $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<$tltm, $tym> {
            indices: Vec<$crate::ecs::entity::index::Index>,
            cursor: usize,
            $tym: parking_lot::MappedRwLockWriteGuard<$tltm, $tym>,
        }

        impl<$tltm, $tym> $name<$tltm, $tym>
        where
            $tym: $crate::ecs::storage::Storage,
        {
            pub(crate) fn new($tym: parking_lot::MappedRwLockWriteGuard<$tltm, $tym>) -> Self {
                $name {
                    indices: $tym.indices().iter().cloned().collect(),
                    cursor: 0,
                    $tym,
                }
            }

            pub fn get(&mut self, index: $crate::ecs::entity::index::Index) -> Option<&$tltm mut $tym::Item> {
                let $tym = self.$tym.get_mut(index)?;

                unsafe { Some(&mut *($tym as *mut _)) }
            }
        }

        impl<$tltm, $tym> ExactSizeIterator for $name<$tltm, $tym>
        where
            $tym: $crate::ecs::storage::Storage,
        {}

        impl<$tltm, $tym> std::iter::FusedIterator for $name<$tltm, $tym>
        where
            $tym: $crate::ecs::storage::Storage,
        {}

        impl<$tltm, $tym> Iterator for $name<$tltm, $tym>
        where
            $tym: $crate::ecs::storage::Storage,
        {
            type Item = ($crate::ecs::entity::index::Index, &$tltm mut $tym::Item);

            fn next(&mut self) -> Option<Self::Item> {
                if self.cursor >= self.indices.len() {
                    return None;
                }

                let idx = self.indices[self.cursor];
                self.cursor += 1;

                unsafe {
                    let $tym = self.$tym.get_unchecked_mut(idx);

                    Some((idx, &mut *($tym as *mut _)))
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining_len = self.indices
                    .len()
                    .saturating_sub(self.cursor);

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
            indices: Vec<$crate::ecs::entity::index::Index>,
            cursor: usize,
            $(
                $ty: parking_lot::MappedRwLockReadGuard<$tlt, $ty>,
            )*
            $(
                $tym: parking_lot::MappedRwLockWriteGuard<$tltm, $tym>,
            )*
        }

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::ecs::storage::Storage,
            )*
            $(
                $tym: $crate::ecs::storage::Storage,
            )*
        {
            pub(crate) fn new($($ty: parking_lot::MappedRwLockReadGuard<$tlt, $ty>,)* $($tym: parking_lot::MappedRwLockWriteGuard<$tltm, $tym>,)*) -> Self {
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

            pub fn get(&mut self, index: $crate::ecs::entity::index::Index) -> Option<($(&$tlt $ty::Item,)* $(&$tltm mut $tym::Item,)*)> {
                $(
                    let $ty = self.$ty.get(index)?;
                )*
                $(
                    let $tym = self.$tym.get_mut(index)?;
                )*

                unsafe { Some(($(& *($ty as *const _),)* $(&mut *($tym as *mut _),)*)) }
            }
        }

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> ExactSizeIterator for $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::ecs::storage::Storage,
            )*
            $(
                $tym: $crate::ecs::storage::Storage,
            )*
        {}

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> std::iter::FusedIterator for $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::ecs::storage::Storage,
            )*
            $(
                $tym: $crate::ecs::storage::Storage,
            )*
        {}

        impl<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*> Iterator for $name<$($tlt,)* $($tltm,)* $($ty,)* $($tym,)*>
        where
            $(
                $ty: $crate::ecs::storage::Storage,
            )*
            $(
                $tym: $crate::ecs::storage::Storage,
            )*
        {
            type Item = ($crate::ecs::entity::index::Index, $(&$tlt $ty::Item,)* $(&$tltm mut $tym::Item,)*);

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

                    Some((idx, $(& *($ty as *const _),)* $(&mut *($tym as *mut _),)*))
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining_len = self.indices
                    .len()
                    .saturating_sub(self.cursor);

                (remaining_len, Some(remaining_len))
            }
        }
    };
}

impl_joined_iter!(RIter, #reads: &'a A);
impl_joined_iter!(WIter, #writes: &'a mut A);

impl_joined_iter_ref!(RIterRef, #reads: Ref<'a, A>);
impl_joined_iter_ref!(WIterRef, #writes: RefMut<'a, A>);

impl_joined_iter_ref!(RRIterRef, #reads: Ref<'a, A>, Ref<'b, B>);
impl_joined_iter_ref!(RWIterRef, #reads: Ref<'a, A>, #writes: RefMut<'b, B>);
impl_joined_iter_ref!(WWIterRef, #writes: RefMut<'a, A>, RefMut<'b, B>);

impl_joined_iter_ref!(RRRIterRef, #reads: Ref<'a, A>, Ref<'b, B>, Ref<'c, C>);
impl_joined_iter_ref!(RRWIterRef, #reads: Ref<'a, A>, Ref<'b, B>, #writes: RefMut<'c, C>);
impl_joined_iter_ref!(RWWIterRef, #reads: Ref<'a, A>, #writes: RefMut<'b, B>, RefMut<'c, C>);
impl_joined_iter_ref!(WWWIterRef, #writes: RefMut<'a, A>, RefMut<'b, B>, RefMut<'c, C>);

pub struct IndexedRIter<'a, S> {
    indices: Vec<super::super::entity::index::Index>,
    cursor: usize,
    storage: &'a S,
}

impl<'a, S> IndexedRIter<'a, S>
where
    S: super::Storage,
{
    pub(crate) fn new(storage: &'a S) -> Self {
        IndexedRIter {
            indices: storage.indices().iter().cloned().collect(),
            cursor: 0,
            storage,
        }
    }
}

impl<'a, S> ExactSizeIterator for IndexedRIter<'a, S> where S: super::Storage {}

impl<'a, S> std::iter::FusedIterator for IndexedRIter<'a, S> where S: super::Storage {}

impl<'a, S> Iterator for IndexedRIter<'a, S>
where
    S: super::Storage,
{
    type Item = (super::super::entity::index::Index, &'a S::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.indices.len() {
            return None;
        }

        let idx = self.indices[self.cursor];
        self.cursor += 1;

        unsafe { Some((idx, self.storage.get_unchecked(idx))) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_len = self.indices.len().saturating_sub(self.cursor);

        (remaining_len, Some(remaining_len))
    }
}

#[cfg(test)]
mod tests {
    use super::super::{vec_storage::VecStorage, Storage};
    use super::*;

    #[test]
    fn size_hint() {
        let mut it = 0..10;
        assert_eq!(it.size_hint(), (10, Some(10)));
        it.next();
        assert_eq!(it.size_hint(), (9, Some(9)));
        it.next();
        assert_eq!(it.size_hint(), (8, Some(8)));
    }

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

    // #[test]
    // fn rr_iter() {
    //     let mut a: VecStorage<usize> = VecStorage::default();
    //     a.insert(0usize, 100usize);
    //     a.insert(1usize, 101usize);
    //     a.insert(2usize, 102usize);

    //     let mut b: VecStorage<usize> = VecStorage::default();
    //     b.insert(0usize, 100usize);
    //     b.insert(1usize, 101usize);
    //     b.insert(2usize, 102usize);

    //     for (ca, cb) in RRIter::new(&a, &b) {
    //         std::convert::identity(ca);
    //         std::convert::identity(cb);
    //     }
    // }
}
