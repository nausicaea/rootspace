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
        }
    };
}

impl_joined_iter!(RRIter, reads: &'a I, &'b J);
impl_joined_iter!(RWIter, reads: &'a I, writes: &'b mut J);
impl_joined_iter!(WWIter, writes: &'a mut I, &'b mut J);
impl_joined_iter!(RRRIter, reads: &'a I, &'b J, &'c K);

