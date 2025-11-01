#![allow(non_snake_case)]

fn intersect_many<T, C>(sets: &[&std::collections::BTreeSet<T>]) -> C
where
    T: std::hash::Hash + Eq + Ord + Clone,
    C: FromIterator<T>,
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
    ($name:ident, #reads: $ty:ident $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<'a, $ty> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $ty: &'a $ty,
        }

        impl<'a, $ty> $name<'a, $ty>
        where
            $ty: $crate::storage::Storage,
        {
            pub fn new($ty: &'a $ty) -> Self {
                $name {
                    indices: $ty.indices().iter().cloned().collect(),
                    cursor: 0,
                    $ty,
                }
            }
        }

        impl<'a, $ty> ExactSizeIterator for $name<'a, $ty>
        where
            $ty: $crate::storage::Storage,
        {}

        impl<'a, $ty> std::iter::FusedIterator for $name<'a, $ty>
        where
            $ty: $crate::storage::Storage,
        {}

        impl<'a, $ty> Iterator for $name<'a, $ty>
        where
            $ty: $crate::storage::Storage,
        {
            type Item = &'a $ty::Item;

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

    ($name:ident, #writes: $tym:ident $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<'a, $tym> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $tym: &'a mut $tym,
        }

        impl<'a, $tym> $name<'a, $tym>
        where
            $tym: $crate::storage::Storage,
        {
            pub fn new($tym: &'a mut $tym) -> Self {
                $name {
                    indices: $tym.indices().iter().cloned().collect(),
                    cursor: 0,
                    $tym,
                }
            }
        }

        impl<'a, $tym> ExactSizeIterator for $name<'a, $tym>
        where
            $tym: $crate::storage::Storage,
        {}

        impl<'a, $tym> std::iter::FusedIterator for $name<'a, $tym>
        where
            $tym: $crate::storage::Storage,
        {}

        impl<'a, $tym> Iterator for $name<'a, $tym>
        where
            $tym: $crate::storage::Storage,
        {
            type Item = &'a mut $tym::Item;

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

    ($name:ident, #reads: $($ty:ident),* $(,)?) => {
        impl_joined_iter!($name, #reads: $($ty),*, #writes: );
    };

    ($name:ident, #writes: $($tym:ident),* $(,)?) => {
        impl_joined_iter!($name, #reads: , #writes: $($tym),*);
    };

    ($name:ident, #reads: $($ty:ident),*, #writes: $($tym:ident),* $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<'a, $($ty,)* $($tym),*> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $(
                $ty: &'a $ty,
            )*
            $(
                $tym: &'a mut $tym,
            )*
        }

        impl<'a, $($ty,)* $($tym),*> $name<'a, $($ty,)* $($tym),*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {
            pub fn new($($ty: &'a $ty,)* $($tym: &'a mut $tym,)*) -> Self {
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

        impl<'a, $($ty,)* $($tym),*> ExactSizeIterator for $name<'a, $($ty,)* $($tym),*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {}

        impl<'a, $($ty,)* $($tym),*> std::iter::FusedIterator for $name<'a, $($ty,)* $($tym),*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {}

        impl<'a, $($ty,)* $($tym),*> Iterator for $name<'a, $($ty,)* $($tym),*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {
            type Item = ($(&'a $ty::Item,)* $(&'a mut $tym::Item,)*);

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
    ($name:ident, #reads: Ref<$ty:ident> $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<'a, $ty> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $ty: parking_lot::MappedRwLockReadGuard<'a, $ty>,
        }

        impl<'a, $ty> $name<'a, $ty>
        where
            $ty: $crate::storage::Storage,
        {
            pub fn new($ty: parking_lot::MappedRwLockReadGuard<'a, $ty>) -> Self {
                $name {
                    indices: $ty.indices().iter().cloned().collect(),
                    cursor: 0,
                    $ty,
                }
            }

            pub fn get(&mut self, index: $crate::entity::index::Index) -> Option<&'a $ty::Item> {
                let $ty = self.$ty.get(index)?;

                unsafe { Some(& *($ty as *const _)) }
            }
        }

        impl<'a, $ty> ExactSizeIterator for $name<'a, $ty>
        where
            $ty: $crate::storage::Storage,
        {}

        impl<'a, $ty> std::iter::FusedIterator for $name<'a, $ty>
        where
            $ty: $crate::storage::Storage,
        {}

        impl<'a, $ty> Iterator for $name<'a, $ty>
        where
            $ty: $crate::storage::Storage,
        {
            type Item = ($crate::entity::index::Index, &'a $ty::Item);

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

    ($name:ident, #writes: RefMut<$tym:ident> $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<'a, $tym> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $tym: parking_lot::MappedRwLockWriteGuard<'a, $tym>,
        }

        impl<'a, $tym> $name<'a, $tym>
        where
            $tym: $crate::storage::Storage,
        {
            pub fn new($tym: parking_lot::MappedRwLockWriteGuard<'a, $tym>) -> Self {
                $name {
                    indices: $tym.indices().iter().cloned().collect(),
                    cursor: 0,
                    $tym,
                }
            }

            pub fn get(&mut self, index: $crate::entity::index::Index) -> Option<&'a mut $tym::Item> {
                let $tym = self.$tym.get_mut(index)?;

                unsafe { Some(&mut *($tym as *mut _)) }
            }
        }

        impl<'a, $tym> ExactSizeIterator for $name<'a, $tym>
        where
            $tym: $crate::storage::Storage,
        {}

        impl<'a, $tym> std::iter::FusedIterator for $name<'a, $tym>
        where
            $tym: $crate::storage::Storage,
        {}

        impl<'a, $tym> Iterator for $name<'a, $tym>
        where
            $tym: $crate::storage::Storage,
        {
            type Item = ($crate::entity::index::Index, &'a mut $tym::Item);

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

    ($name:ident, #reads: $(Ref<$ty:ident>),* $(,)?) => {
        impl_joined_iter_ref!($name, #reads: $(Ref<$ty>),*, #writes: );
    };

    ($name:ident, #writes: $(RefMut<$tym:ident>),* $(,)?) => {
        impl_joined_iter_ref!($name, #reads: , #writes: $(RefMut<$tym>),*);
    };

    ($name:ident, #reads: $(Ref<$ty:ident>),*, #writes: $(RefMut<$tym:ident>),* $(,)?) => {
        // An iterator that allows iterating over the intersection of multiple components.
        // In other words, this iterator will only go over those entities that have all of the
        // requested component types.
        pub struct $name<'a, $($ty,)* $($tym),*> {
            indices: Vec<$crate::entity::index::Index>,
            cursor: usize,
            $(
                $ty: parking_lot::MappedRwLockReadGuard<'a, $ty>,
            )*
            $(
                $tym: parking_lot::MappedRwLockWriteGuard<'a, $tym>,
            )*
        }

        impl<'a, $($ty,)* $($tym),*> $name<'a, $($ty,)* $($tym),*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {
            pub fn new($($ty: parking_lot::MappedRwLockReadGuard<'a, $ty>,)* $($tym: parking_lot::MappedRwLockWriteGuard<'a, $tym>,)*) -> Self {
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

            pub fn get(&mut self, index: $crate::entity::index::Index) -> Option<($(&'a $ty::Item,)* $(&'a mut $tym::Item,)*)> {
                $(
                    let $ty = self.$ty.get(index)?;
                )*
                $(
                    let $tym = self.$tym.get_mut(index)?;
                )*

                unsafe { Some(($(& *($ty as *const _),)* $(&mut *($tym as *mut _),)*)) }
            }
        }

        impl<'a, $($ty,)* $($tym),*> ExactSizeIterator for $name<'a, $($ty,)* $($tym),*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {}

        impl<'a, $($ty,)* $($tym),*> std::iter::FusedIterator for $name<'a, $($ty,)* $($tym),*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {}

        impl<'a, $($ty,)* $($tym),*> Iterator for $name<'a, $($ty,)* $($tym),*>
        where
            $(
                $ty: $crate::storage::Storage,
            )*
            $(
                $tym: $crate::storage::Storage,
            )*
        {
            type Item = ($crate::entity::index::Index, $(&'a $ty::Item,)* $(&'a mut $tym::Item,)*);

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

impl_joined_iter!(RIter, #reads: A);
impl_joined_iter!(WIter, #writes: A);

impl_joined_iter_ref!(RIterRef, #reads: Ref<A>);
impl_joined_iter_ref!(WIterRef, #writes: RefMut<A>);

impl_joined_iter_ref!(RRIterRef, #reads: Ref<A>, Ref<B>);
impl_joined_iter_ref!(RWIterRef, #reads: Ref<A>, #writes: RefMut<B>);
impl_joined_iter_ref!(WWIterRef, #writes: RefMut<A>, RefMut<B>);

impl_joined_iter_ref!(RRRIterRef, #reads: Ref<A>, Ref<B>, Ref<C>);
impl_joined_iter_ref!(RRWIterRef, #reads: Ref<A>, Ref<B>, #writes: RefMut<C>);
impl_joined_iter_ref!(RWWIterRef, #reads: Ref<A>, #writes: RefMut<B>, RefMut<C>);
impl_joined_iter_ref!(WWWIterRef, #writes: RefMut<A>, RefMut<B>, RefMut<C>);

pub struct IndexedRIter<'a, S> {
    indices: Vec<super::super::entity::index::Index>,
    cursor: usize,
    storage: &'a S,
}

impl<'a, S> IndexedRIter<'a, S>
where
    S: super::Storage,
{
    pub fn new(storage: &'a S) -> Self {
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
    type Item = (crate::entity::index::Index, &'a S::Item);

    #[cfg_attr(test, mutants::skip)] // Mutations cause hangs
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
    use super::{
        super::{Storage, vec_storage::VecStorage},
        *,
    };

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
