use crate::{
    archetype::Archetype,
    prelude::ComponentFactory,
    world::storage::arch_storage::{ArchStorage, ArchStorageIndex},
};
use std::marker::PhantomData;
use worlds_derive::all_tuples;

pub struct Not<T: ArchFilter>(PhantomData<T>);

pub struct Or<T: ArchFilter>(PhantomData<T>);

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

unsafe impl<Q: ArchFilter> ArchFilter for Not<Q> {
    unsafe fn filter<'a>(
        arch_storage: *const ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> impl FilterResult {
        !Q::filter(arch_storage, index, comp_factory).all()
    }
}

unsafe impl<Q: ArchFilter> ArchFilter for Or<Q> {
    unsafe fn filter<'a>(
        arch_storage: *const ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> impl FilterResult {
        Q::filter(arch_storage, index, comp_factory).any()
    }
}

unsafe impl<A: Archetype> ArchFilter for Contains<A> {
    unsafe fn filter<'a>(
        arch_storage: *const ArchStorage,
        _index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> impl FilterResult {
        (*arch_storage).contains_archetype::<A>(comp_factory)
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
