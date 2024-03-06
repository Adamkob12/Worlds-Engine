use crate::{
    prelude::ComponentFactory,
    world::storage::arch_storage::{ArchStorage, ArchStorageIndex},
};

use super::arch_query::ArchQuery;

#[doc(hidden)]
pub trait FilteringValue {}

impl FilteringValue for bool {}

pub unsafe trait QueryFilter: ArchQuery
where
    for<'a> Self::Item<'a>: FilteringValue,
{
    /// # Safety
    ///   1) The caller must ensure that the [`ArchStorageIndex`] is withing the bounds of the [`ArchStorage`]
    /// (as specified in [`ArchStorage::get_component_unchecked`]).
    ///   2) The caller must ensure that the raw pointer to [`ArchStorage`] is valid, and usable.
    unsafe fn filter<'a>(
        arch_storage: *const ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> Self::Item<'a> {
        Self::fetch(arch_storage as *mut ArchStorage, index, comp_factory)
    }
}

unsafe impl<T: ArchQuery> QueryFilter for T where for<'a> T::Item<'a>: FilteringValue {}
