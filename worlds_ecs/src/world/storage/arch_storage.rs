use crate::{
    archetype::{Archetype, MAX_COMPS_PER_ARCH},
    prelude::{Bundle, ComponentFactory, ComponentId},
    storage::blob_vec::BlobVec,
    utils::prime_key::PrimeArchKey,
};
use bevy_ptr::{OwningPtr, Ptr, PtrMut};
use smallvec::SmallVec;
use std::collections::HashMap;

/// Used to index an [`ArchStorage`]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ArchStorageIndex(usize);

/// A data-structure that stores the data of an archetype (a.k.a [`Bundle`]).
pub struct ArchStorage {
    /// By indexing this list using [`ComponentId::id`], we get the index to the component's storage
    /// in the `comp_storage` field.
    comp_indexes: HashMap<ComponentId, usize>,
    /// The raw storage of the components.
    comp_storage: SmallVec<[BlobVec; MAX_COMPS_PER_ARCH]>,
    /// The [`PrimeArchKey`] of the archetype stored here.
    prime_key: PrimeArchKey,
    /// The amount of bundles stored
    len: usize,
}

impl ArchStorage {
    /// Create a new [`ArchStorage`] for an archetype
    pub fn new<A: Archetype>(comp_factory: &ComponentFactory) -> Option<ArchStorage> {
        let arch_info = A::arch_info(comp_factory)?;
        let components = arch_info.component_ids();
        let mut comp_storage = SmallVec::new();
        let mut comp_indexes = HashMap::with_capacity(MAX_COMPS_PER_ARCH);
        for (i, comp_id) in components.iter().enumerate() {
            // SAFETY: the safety is dependant on whether each of the archetype's components'
            // [`DataInfo`] that is stored internally in the `ComponentFactory` matches their type.
            comp_storage.push(unsafe { comp_factory.new_component_storage(*comp_id)? });
            comp_indexes.insert(*comp_id, i);
        }
        Some(ArchStorage {
            comp_indexes,
            prime_key: arch_info.prime_key(),
            comp_storage,
            len: 0,
        })
    }

    /// The amount of bundles stored in [`Self`]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return `true` if there is nothing stored here. else `false`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Store a [`Bundle`] of components with a matching archetype in this storage.
    pub fn store_bundle<B: Bundle + Archetype>(
        &mut self,
        comp_factory: &ComponentFactory,
        bundle: B,
    ) -> Option<ArchStorageIndex> {
        B::prime_key(comp_factory)?
            .is_exact_archetype(self.prime_key)
            // SAFETY: We checked that the archetypes are matching
            .then_some(unsafe { self.store_bundle_unchecked(comp_factory, bundle) })
    }

    /// Store a [`Bundle`] of components in this storage, without checking whether the archetypes are matching.
    ///
    /// # Safety
    /// The caller must ensure that the bundle's archetypes matches the archetype that is stored in this storage.
    pub unsafe fn store_bundle_unchecked<B: Bundle>(
        &mut self,
        comp_factory: &ComponentFactory,
        bundle: B,
    ) -> ArchStorageIndex {
        bundle.raw_components_scope(comp_factory, &mut |comp_id, raw_comp| {
            self.store_component_unchecked(comp_id, raw_comp)
        });
        self.len += 1;
        ArchStorageIndex(self.len - 1)
    }

    /// Store a single component in its matching [`BlobVec`].
    /// # Safety
    /// The caller must ensure that:
    ///     - All the other components will also be stored in the same "go" (no [`BlobVec`]) in
    ///        `Self::comp_storage` will have a different length of the others.
    ///     - The raw data (`raw_comp`) matches the component's `Layout` (the same safety requirements
    ///       that are needed when using [`BlobVec::push`])
    ///     - The component is part of the archetypes (Components of this type are stored in [`Self`])
    unsafe fn store_component_unchecked(&mut self, comp_id: ComponentId, raw_comp: OwningPtr<'_>) {
        self.comp_storage[*self.comp_indexes.get(&comp_id).unwrap_unchecked()].push(raw_comp)
    }

    /// Get a type-erased reference to a pointer, from its index and [`ComponentId`].
    pub fn get_component(&self, index: ArchStorageIndex, comp_id: ComponentId) -> Option<Ptr<'_>> {
        (index.0 < self.len).then_some(
            // SAFETY: We ensured that `index < self.len`.
            unsafe { self.comp_storage[*self.comp_indexes.get(&comp_id)?].get_unchecked(index.0) },
        )
    }

    /// Get a type-erased reference to a pointer, from its index and [`ComponentId`].
    ///
    /// # Safety
    /// The caller must ensure that the component matching the given [`ComponentId`] is indeed
    /// stored in [`Self`], and that `index < self.len()`.
    pub unsafe fn get_component_unchecked(
        &self,
        index: ArchStorageIndex,
        comp_id: ComponentId,
    ) -> Ptr<'_> {
        self.comp_storage[*self.comp_indexes.get(&comp_id).unwrap_unchecked()]
            .get_unchecked(index.0)
    }

    /// Get a type-erased mutable reference to a pointer, from its index and [`ComponentId`].
    pub fn get_component_mut(
        &mut self,
        index: ArchStorageIndex,
        comp_id: ComponentId,
    ) -> Option<PtrMut<'_>> {
        (index.0 < self.len).then_some(
            // SAFETY: We ensured that `index < self.len`.
            unsafe {
                self.comp_storage[*self.comp_indexes.get(&comp_id)?].get_mut_unchecked(index.0)
            },
        )
    }

    /// Get a type-erased mutable reference to a pointer, from its index and [`ComponentId`].
    ///
    /// # Safety
    /// The caller must ensure that the component matching the given [`ComponentId`] is indeed
    /// stored in [`Self`], and that `index < self.len()`.
    pub unsafe fn get_component_mut_unchecked(
        &mut self,
        index: ArchStorageIndex,
        comp_id: ComponentId,
    ) -> PtrMut<'_> {
        self.comp_storage[*self.comp_indexes.get(&comp_id).unwrap_unchecked()]
            .get_mut_unchecked(index.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ArchStorage;
    use super::ArchStorageIndex;
    use crate::prelude::*;

    #[derive(Component)]
    struct A(usize);
    #[derive(Component)]
    struct B([usize; 2]);
    #[derive(Component)]
    struct C([u8; 3]);

    #[test]
    fn test_component_storage() {
        let mut comp_factory = ComponentFactory::default();

        comp_factory.register_component::<A>(); // will have `ComponentId` 0
        comp_factory.register_component::<B>(); // will have `ComponentId` 1
        comp_factory.register_component::<C>(); // will have `ComponentId` 2

        let mut abc_storage = ArchStorage::new::<(A, B, C)>(&comp_factory).unwrap();
        // let mut ab_storage = ArchStorage::new::<(A, B)>(&comp_factory).unwrap();
        // let mut bc_storage = ArchStorage::new::<(B, C)>(&comp_factory).unwrap();
        // let mut ac_storage = ArchStorage::new::<(A, C)>(&comp_factory).unwrap();
        // let mut a_storage = ArchStorage::new::<A>(&comp_factory).unwrap();
        // let mut b_storage = ArchStorage::new::<B>(&comp_factory).unwrap();
        // let mut c_storage = ArchStorage::new::<C>(&comp_factory).unwrap();

        assert_eq!(abc_storage.len(), 0);

        assert_eq!(
            abc_storage
                .store_bundle(&comp_factory, (A(0), B([1; 2]), C([255; 3])))
                .unwrap()
                .0,
            0
        );
        assert_eq!(
            abc_storage
                .store_bundle(&comp_factory, (A(1), B([10; 2]), C([255; 3])))
                .unwrap()
                .0,
            1
        );
        assert_eq!(
            abc_storage
                .store_bundle(&comp_factory, (A(2), B([100; 2]), C([255; 3])))
                .unwrap()
                .0,
            2
        );
        assert_eq!(
            abc_storage
                .store_bundle(&comp_factory, (A(3), B([1000; 2]), C([255; 3])))
                .unwrap()
                .0,
            3
        );

        assert_eq!(abc_storage.len(), 4);

        // ~~~~~~~~~~~~~~~~~~~~~~
        //
        // TEST READING COMPONENTS
        //
        // ~~~~~~~~~~~~~~~~~~~~~~

        unsafe {
            assert_eq!(
                abc_storage
                    .get_component(ArchStorageIndex(0), ComponentId::new(0))
                    .unwrap()
                    .deref::<A>()
                    .0,
                0
            );
            assert_eq!(
                abc_storage
                    .get_component(ArchStorageIndex(1), ComponentId::new(0))
                    .unwrap()
                    .deref::<A>()
                    .0,
                1
            );
            assert_eq!(
                abc_storage
                    .get_component_unchecked(ArchStorageIndex(2), ComponentId::new(0))
                    .deref::<A>()
                    .0,
                2
            );
            assert_eq!(
                abc_storage
                    .get_component_unchecked(ArchStorageIndex(3), ComponentId::new(0))
                    .deref::<A>()
                    .0,
                3
            );
        }

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        //
        // TEST WRITING / CHANGING COMPONENTS
        //
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

        unsafe {
            abc_storage
                .get_component_mut(ArchStorageIndex(0), ComponentId::new(0))
                .unwrap()
                .deref_mut::<A>()
                .0 *= 10;
            abc_storage
                .get_component_mut(ArchStorageIndex(1), ComponentId::new(0))
                .unwrap()
                .deref_mut::<A>()
                .0 *= 10;
            abc_storage
                .get_component_mut(ArchStorageIndex(2), ComponentId::new(0))
                .unwrap()
                .deref_mut::<A>()
                .0 *= 10;
            abc_storage
                .get_component_mut(ArchStorageIndex(3), ComponentId::new(0))
                .unwrap()
                .deref_mut::<A>()
                .0 *= 10;
        }

        unsafe {
            assert_eq!(
                abc_storage
                    .get_component(ArchStorageIndex(0), ComponentId::new(0))
                    .unwrap()
                    .deref::<A>()
                    .0,
                0
            );
            assert_eq!(
                abc_storage
                    .get_component(ArchStorageIndex(1), ComponentId::new(0))
                    .unwrap()
                    .deref::<A>()
                    .0,
                10
            );
            assert_eq!(
                abc_storage
                    .get_component_unchecked(ArchStorageIndex(2), ComponentId::new(0))
                    .deref::<A>()
                    .0,
                20
            );
            assert_eq!(
                abc_storage
                    .get_component_unchecked(ArchStorageIndex(3), ComponentId::new(0))
                    .deref::<A>()
                    .0,
                30
            );
        }

        //
    }
}
