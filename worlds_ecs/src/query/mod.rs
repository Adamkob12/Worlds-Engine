#![allow(missing_docs)] // TODO: Remove

use worlds_derive::all_tuples;

use crate::{
    prelude::{Component, ComponentFactory},
    utils::prime_key::PrimeArchKey,
    world::storage::{
        arch_storage::{ArchStorage, ArchStorageIndex},
        storages::ArchStorages,
    },
};

pub unsafe trait ComponentQuery {
    type Item<'a>;
    fn matching_prime_arch_key(comp_factory: &ComponentFactory) -> PrimeArchKey;
    /// # Safety
    ///     - The caller must ensure that the [`ArchStorageIndex`] is withing the bounds of the [`ArchStorage`]
    /// (as specified in [`ArchStorage::get_component_unchecked`]).
    ///     - The caller must ensure that the raw pointer to [`ArchStorage`] is valid, and usable.
    unsafe fn fetch<'a>(
        arch_storage: *mut ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> Self::Item<'a>;

    /// # Safety
    ///     - The caller must ensure that the raw pointer to [`ArchStorages`] is valid, and usable.
    unsafe fn iter_query_matches<'a>(
        arch_storages: *mut ArchStorages,
        comp_factory: &'a ComponentFactory,
    ) -> impl Iterator<Item = Self::Item<'a>> + 'a {
        let pkey = Self::matching_prime_arch_key(comp_factory);
        (*arch_storages)
            .iter_storages_with_matching_archetype_mut(pkey)
            .map(|arch_storage| {
                arch_storage
                    .iter_indices()
                    // SAFETY: The index must be in bounds because it came from the storage itself.
                    .map(|index| unsafe { Self::fetch(arch_storage, index, comp_factory) })
            })
            .flatten()
    }
}

unsafe impl<C: Component> ComponentQuery for &C {
    type Item<'a> = &'a C;

    unsafe fn fetch<'a>(
        arch_storage: *mut ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> Self::Item<'a> {
        (*arch_storage)
            .get_component_unchecked(
                index,
                comp_factory
                    .get_component_id::<C>()
                    .expect("Can't query unregistered component"),
            )
            .deref::<C>()
    }

    fn matching_prime_arch_key(comp_factory: &ComponentFactory) -> PrimeArchKey {
        comp_factory
            .get_component_id::<C>()
            .expect("Can't query unregistered component")
            .prime_key()
    }
}

unsafe impl<C: Component> ComponentQuery for &mut C {
    type Item<'a> = &'a mut C;

    unsafe fn fetch<'a>(
        arch_storage: *mut ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> Self::Item<'a> {
        (*arch_storage)
            .get_component_mut_unchecked(
                index,
                comp_factory
                    .get_component_id::<C>()
                    .expect("Can't query unregistered component"),
            )
            .deref_mut::<C>()
    }

    fn matching_prime_arch_key(comp_factory: &ComponentFactory) -> PrimeArchKey {
        comp_factory
            .get_component_id::<C>()
            .expect("Can't query unregistered component")
            .prime_key()
    }
}

macro_rules! impl_comp_query_for_tuple {
    ($($name:ident),*) => {
        #[allow(non_snake_case, unused)]
        unsafe impl<$($name: ComponentQuery),*> ComponentQuery for ($($name,)*) {
            type Item<'a> = ($($name::Item<'a>,)*);

            unsafe fn fetch<'a>(
                arch_storage: *mut ArchStorage,
                index: ArchStorageIndex,
                comp_factory: &'a ComponentFactory,
            ) -> Self::Item<'a> {
                ($($name::fetch(arch_storage, index, comp_factory),)*)
            }

            fn matching_prime_arch_key(comp_factory: &ComponentFactory) -> PrimeArchKey {
                let mut pkey = PrimeArchKey::IDENTITY;
                $(pkey.merge_with($name::matching_prime_arch_key(comp_factory));)*
                pkey
            }
        }
    };
}

all_tuples!(impl_comp_query_for_tuple, 0, 12, B);
