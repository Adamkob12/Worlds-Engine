use super::query_filter::{ArchFilter, FilterResult};
use crate::{
    entity::EntityId,
    prelude::{Component, ComponentFactory},
    utils::prime_key::PrimeArchKey,
    world::storage::{ArchEntityStorage, arch_storage::ArchStorageIndex, storages::ArchStorages},
};
use worlds_derive::all_tuples;

pub unsafe trait ArchQuery {
    type Item<'a>;
    #[inline]
    fn merge_prime_arch_key_with(_pkey: &mut PrimeArchKey, _comp_factory: &ComponentFactory) {}
    /// # Safety
    ///   1) The caller must ensure that the [`ArchStorageIndex`] is withing the bounds of the [`ArchStorage`]
    /// (as specified in [`ArchStorage::get_component_unchecked`]).
    ///   2) The caller must ensure that the raw pointer to [`ArchStorage`] is valid, and usable.
    unsafe fn fetch(
        arch_storage: *mut ArchEntityStorage,
        index: ArchStorageIndex,
        comp_factory: &ComponentFactory,
    ) -> Self::Item<'_>;

    /// # Safety
    ///  1) The caller must ensure that the raw pointer to [`ArchStorages`] is valid, and usable.
    unsafe fn iter_query_matches<'a>(
        arch_storages: *mut ArchStorages,
        comp_factory: &'a ComponentFactory,
    ) -> impl Iterator<Item = Self::Item<'a>> + 'a {
        let mut pkey = PrimeArchKey::IDENTITY;
        Self::merge_prime_arch_key_with(&mut pkey, comp_factory);
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

    /// # Safety
    ///  1) The caller must ensure that the raw pointer to [`ArchStorages`] is valid, and usable.
    unsafe fn iter_filtered_query_matches<F: ArchFilter>(
        arch_storages: *mut ArchStorages,
        comp_factory: &ComponentFactory,
    ) -> impl Iterator<Item=Self::Item<'_>>  {
        let mut pkey = PrimeArchKey::IDENTITY;
        Self::merge_prime_arch_key_with(&mut pkey, comp_factory);
        (*arch_storages)
            .iter_storages_with_matching_archetype_mut(pkey)
            .map(|arch_storage| {
                arch_storage
                    .iter_indices()
                    // SAFETY: The index must be in bounds because it came from the storage itself.
                    .filter_map(|index| unsafe {
                        F::filter(arch_storage, index, comp_factory)
                            .collapse()
                            .then_some(Self::fetch(arch_storage, index, comp_factory))
                    })
            })
            .flatten()
    }
}

unsafe impl<C: Component> ArchQuery for &C {
    type Item<'a> = &'a C;

    unsafe fn fetch(
        arch_storage: *mut ArchEntityStorage,
        index: ArchStorageIndex,
        comp_factory: &ComponentFactory,
    ) -> Self::Item<'_> {
        (*arch_storage)
            .get_component_unchecked(
                index,
                comp_factory
                    .get_component_id::<C>()
                    .expect("Can't query unregistered component"),
            )
            .deref::<C>()
    }

    fn merge_prime_arch_key_with(pkey: &mut PrimeArchKey, comp_factory: &ComponentFactory) {
        pkey.merge_with_but_panic_if_already_merged(
            comp_factory
                .get_component_id::<C>()
                .expect("Can't query unregistered component")
                .prime_key(),
            "Can't query duplicate components",
        )
    }
}

unsafe impl<C: Component> ArchQuery for &mut C {
    type Item<'a> = &'a mut C;

    unsafe fn fetch(
        arch_storage: *mut ArchEntityStorage,
        index: ArchStorageIndex,
        comp_factory: &ComponentFactory,
    ) -> Self::Item<'_> {
        (*arch_storage)
            .get_component_mut_unchecked(
                index,
                comp_factory
                    .get_component_id::<C>()
                    .expect("Can't query unregistered component"),
            )
            .deref_mut::<C>()
    }

    fn merge_prime_arch_key_with(pkey: &mut PrimeArchKey, comp_factory: &ComponentFactory) {
        pkey.merge_with_but_panic_if_already_merged(
            comp_factory
                .get_component_id::<C>()
                .expect("Can't query unregistered component")
                .prime_key(),
            "Can't query duplicate components",
        )
    }
}

unsafe impl<C: Component> ArchQuery for Option<&mut C> {
    type Item<'a> = Option<&'a mut C>;

    unsafe fn fetch(
        arch_storage: *mut ArchEntityStorage,
        index: ArchStorageIndex,
        comp_factory: &ComponentFactory,
    ) -> Self::Item<'_> {
        (*arch_storage)
            .get_component_mut(
                index,
                comp_factory
                    .get_component_id::<C>()
                    .expect("Can't query unregistered component"),
            )
            .map(|c| c.deref_mut::<C>())
    }
}

unsafe impl<C: Component> ArchQuery for Option<&C> {
    type Item<'a> = Option<&'a C>;

    unsafe fn fetch(
        arch_storage: *mut ArchEntityStorage,
        index: ArchStorageIndex,
        comp_factory: &ComponentFactory,
    ) -> Self::Item<'_> {
        (*arch_storage)
            .get_component(
                index,
                comp_factory
                    .get_component_id::<C>()
                    .expect("Can't query unregistered component"),
            )
            .map(|c| c.deref::<C>())
    }
}

unsafe impl ArchQuery for EntityId {
    type Item<'a> = EntityId;

    unsafe fn fetch(
        arch_storage: *mut ArchEntityStorage,
        index: ArchStorageIndex,
        _comp_factory: &ComponentFactory,
    ) -> Self::Item<'_> {
        unsafe { (*arch_storage).get_entity_at_unchecked(index) }
    }
}

//
//
//
//
//

macro_rules! impl_comp_query_for_tuple {
    ($($name:ident),*) => {
        #[allow(non_snake_case, unused)]
        unsafe impl<$($name: ArchQuery),*> ArchQuery for ($($name,)*) {
            type Item<'a> = ($($name::Item<'a>,)*);

            unsafe fn fetch<'a>(
                arch_storage: *mut ArchEntityStorage,
                index: ArchStorageIndex,
                comp_factory: &'a ComponentFactory,
            ) -> Self::Item<'a> {
                unsafe { ($($name::fetch(arch_storage, index, comp_factory),)*) }
            }

            fn merge_prime_arch_key_with(pkey: &mut PrimeArchKey, comp_factory: &ComponentFactory) {
                $($name::merge_prime_arch_key_with(pkey, comp_factory);)*
            }
        }
    };
}

all_tuples!(impl_comp_query_for_tuple, 0, 12, Q);
