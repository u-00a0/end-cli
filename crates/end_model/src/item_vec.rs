use std::ops::{Index, IndexMut};

use generativity::Id;

use crate::{Catalog, ItemId};

/// Dense vector indexed by [`ItemId`].
///
/// The vector length is tied to a source [`Catalog`], and indexing is guarded by
/// the same branded lifetime as [`ItemId`].
#[derive(Debug, Clone)]
pub struct ItemVec<'id, T> {
    values: Box<[T]>,
    _brand: Id<'id>,
}

impl<'id, T: Clone> ItemVec<'id, T> {
    pub fn filled(catalog: &Catalog<'id>, value: T) -> Self {
        Self {
            values: vec![value; catalog.items().len()].into_boxed_slice(),
            _brand: catalog.brand(),
        }
    }
}

impl<'id, T> ItemVec<'id, T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.values.iter()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.values
    }
}

impl<'id, T> Index<ItemId<'id>> for ItemVec<'id, T> {
    type Output = T;

    fn index(&self, index: ItemId<'id>) -> &Self::Output {
        // SAFETY: `ItemId<'id>` and `Catalog<'id>` share the same `generativity::Id` brand.
        // `filled` allocates one slot per catalog item in dense id order.
        unsafe { self.values.get_unchecked(index.index()) }
    }
}

impl<'id, T> IndexMut<ItemId<'id>> for ItemVec<'id, T> {
    fn index_mut(&mut self, index: ItemId<'id>) -> &mut Self::Output {
        // SAFETY: same reasoning as `Index::index`.
        unsafe { self.values.get_unchecked_mut(index.index()) }
    }
}
