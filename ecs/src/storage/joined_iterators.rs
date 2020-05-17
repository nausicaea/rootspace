#![allow(non_snake_case)]

fn intersect_many<T, C>(sets: &[&std::collections::HashSet<T>]) -> C
where
    T: std::hash::Hash + Eq + Clone,
    C: std::iter::FromIterator<T>,
{
    let shortest_set = sets
        .iter()
        .min_by(|&&a, &&b| a.len().cmp(&b.len()));

    shortest_set
        .iter()
        .flat_map(|&&s| s.iter())
        .filter(|&k| sets.iter().all(|&s| s.contains(k)))
        .cloned()
        .collect()
}

macro_rules! impl_joined_iter {
    ($name:ident, reads: $(&$tlt:lifetime $ty:ident),*) => {
        impl_joined_iter!($name, reads: $(&$tlt $ty),*, writes: );
    };

    ($name:ident, writes: $(&$tltm:lifetime mut $tym:ident),*) => {
        impl_joined_iter!($name, reads: , writes: $(&$tltm mut $tym),*);
    };

    ($name:ident, reads: $(&$tlt:lifetime $ty:ident),*, writes: $(&$tltm:lifetime mut $tym:ident),*) => {
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
                    indices: intersect_many(&[$($ty.index(),)* $($tym.index(),)*]),
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

impl_joined_iter!(RRIter, reads: &'a I, &'b J);
impl_joined_iter!(RWIter, reads: &'a I, writes: &'b mut J);
impl_joined_iter!(WWIter, writes: &'a mut I, &'b mut J);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{Storage, vec_storage::VecStorage};

    #[test]
    fn rr_iter() {
        let mut a: VecStorage<usize> = VecStorage::default();
        a.insert(0usize, 100usize);
        a.insert(1usize, 101usize);
        a.insert(2usize, 102usize);

        let mut b: VecStorage<String> = VecStorage::default();
        b.insert(0usize, "0".into());
        b.insert(2usize, "2".into());

        for (ca, cb) in RRIter::new(&a, &b) {
            eprintln!("ca: {}, cb: {}", ca, cb);
        }
    }

    #[test]
    fn rw_iter() {
        let mut a: VecStorage<usize> = VecStorage::default();
        a.insert(0usize, 100usize);
        a.insert(1usize, 101usize);
        a.insert(2usize, 102usize);

        let mut b: VecStorage<String> = VecStorage::default();
        b.insert(0usize, "0".into());
        b.insert(2usize, "2".into());

        for (ca, cb) in RWIter::new(&a, &mut b) {
            eprintln!("ca: {}, cb: {}", ca, cb);
        }
    }

    #[test]
    fn ww_iter() {
        let mut a: VecStorage<usize> = VecStorage::default();
        a.insert(0usize, 100usize);
        a.insert(1usize, 101usize);
        a.insert(2usize, 102usize);

        let mut b: VecStorage<String> = VecStorage::default();
        b.insert(0usize, "0".into());
        b.insert(2usize, "2".into());

        for (ca, cb) in WWIter::new(&mut a, &mut b) {
            eprintln!("ca: {}, cb: {}", ca, cb);
        }
    }
}
