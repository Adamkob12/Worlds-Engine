use crate::{
    archetype::Archetype,
    prelude::ComponentFactory,
    world::storage::arch_storage::{ArchStorage, ArchStorageIndex},
};
use std::marker::PhantomData;
use worlds_derive::all_tuples;

use super::arch_query::ArchQuery;

pub struct Not<T>(PhantomData<T>);

pub struct Or<T>(PhantomData<T>);

pub struct Contains<T>(PhantomData<T>);

pub unsafe trait ArchFilter
where
    Self: Sized,
{
    /// # Safety
    ///   1) The caller must ensure that the [`ArchStorageIndex`] is withing the bounds of the [`ArchStorage`]
    /// (as specified in [`ArchStorage::get_component_unchecked`]).
    ///   2) The caller must ensure that the raw pointer to [`ArchStorage`] is valid, and usable.
    unsafe fn filter<'a>(
        arch_storage: *const ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> impl FilterResult;
}

#[doc(hidden)]
pub trait FilterResult
where
    Self: Sized,
{
    fn collapse(self) -> bool {
        self.all()
    }
    fn all(self) -> bool;
    fn any(self) -> bool;
}

impl FilterResult for bool {
    fn all(self) -> bool {
        self
    }

    fn any(self) -> bool {
        self
    }
}

unsafe impl<Q: ArchFilter> ArchQuery for Not<Q> {
    type Item<'a> = bool;

    unsafe fn fetch<'a>(
        arch_storage: *mut ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> bool {
        !Q::filter(arch_storage, index, comp_factory).collapse()
    }
}

unsafe impl<Q: ArchFilter> ArchQuery for Or<Q> {
    type Item<'a> = bool;

    unsafe fn fetch<'a>(
        arch_storage: *mut ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> bool {
        Q::filter(arch_storage, index, comp_factory).any()
    }
}

unsafe impl<A: Archetype> ArchQuery for Contains<A> {
    type Item<'a> = bool;

    unsafe fn fetch<'a>(
        arch_storage: *mut ArchStorage,
        _index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> bool {
        (*arch_storage).contains_archetype::<A>(comp_factory)
    }

    fn merge_prime_arch_key_with(
        _pkey: &mut crate::utils::prime_key::PrimeArchKey,
        _comp_factory: &ComponentFactory,
    ) {
        // No need, because this doesn't change the archetype.
    }
}

unsafe impl<Q: ArchQuery> ArchFilter for Q
where
    for<'a> Q::Item<'a>: FilterResult,
{
    unsafe fn filter<'a>(
        arch_storage: *const ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> impl FilterResult {
        Q::fetch(arch_storage as *mut ArchStorage, index, comp_factory)
    }
}

macro_rules! impl_filtering_value_for_tuple {
    ($($name:ident),*) => {
        #[allow(non_snake_case, unused)]
        impl<$($name: FilterResult),*> FilterResult for ($($name,)*) {
            fn all(self) -> bool {
                let ($($name,)*) = self;
                true $(&& $name.all())*
            }

            fn any(self) -> bool {
                let ($($name,)*) = self;
                false $(|| $name.any())*
            }
        }
    };
}

all_tuples!(impl_filtering_value_for_tuple, 0, 12, F);
