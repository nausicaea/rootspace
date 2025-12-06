use super::{super::entity::index::Index, Storage};

#[derive(Debug)]
pub enum Entry<'a, T: 'a, S: Storage<Item = T>> {
    Occupied(&'a mut S, Index),
    Vacant(&'a mut S, Index),
}

impl<'a, T: 'a, S: Storage<Item = T>> Entry<'a, T, S> {
    #[must_use]
    pub fn index(&self) -> &Index {
        match self {
            Entry::Occupied(_, i) => i,
            Entry::Vacant(_, i) => i,
        }
    }

    pub fn or_insert(self, default: T) -> &'a mut T {
        match self {
            Entry::Vacant(s, i) => {
                s.insert(i, default);
                unsafe { s.get_unchecked_mut(i) }
            }
            Entry::Occupied(s, i) => unsafe { s.get_unchecked_mut(i) },
        }
    }

    pub fn or_insert_with<F: FnOnce() -> T>(self, f: F) -> &'a mut T {
        match self {
            Entry::Vacant(s, i) => {
                s.insert(i, f());
                unsafe { s.get_unchecked_mut(i) }
            }
            Entry::Occupied(s, i) => unsafe { s.get_unchecked_mut(i) },
        }
    }

    pub fn or_insert_with_key<F: FnOnce(Index) -> T>(self, f: F) -> &'a mut T {
        match self {
            Entry::Vacant(s, i) => {
                s.insert(i, f(i));
                unsafe { s.get_unchecked_mut(i) }
            }
            Entry::Occupied(s, i) => unsafe { s.get_unchecked_mut(i) },
        }
    }

    pub fn and_modify<F: FnOnce(&mut T)>(self, f: F) -> Self {
        match self {
            Entry::Vacant(s, i) => Entry::Vacant(s, i),
            Entry::Occupied(s, i) => {
                f(unsafe { s.get_unchecked_mut(i) });
                Entry::Occupied(s, i)
            }
        }
    }
}

impl<'a, T: Default + 'a, S: Storage<Item = T>> Entry<'a, T, S> {
    #[must_use]
    pub fn or_default(self) -> &'a mut T {
        match self {
            Entry::Vacant(s, i) => {
                s.insert(i, Default::default());
                unsafe { s.get_unchecked_mut(i) }
            }
            Entry::Occupied(s, i) => unsafe { s.get_unchecked_mut(i) },
        }
    }
}
