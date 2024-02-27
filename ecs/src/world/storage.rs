use std::collections::HashMap;

use bevy_ptr::{OwningPtr, Ptr};
use smallvec::SmallVec;

use crate::{
    archetype::{Archetype, MAX_COMPS_PER_ARCH},
    prelude::{Bundle, Component, ComponentFactory, ComponentId},
    storage::blob_vec::BlobVec,
    utils::prime_key::PrimeArchKey,
};

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
        for (i, comp_id) in components.into_iter().enumerate() {
            // SAFETY: the safety is dependant on whether each of the archetype's components'
            // `DataInfo` that is stored internally in the `ComponentFactory` matches their type.
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

    /// Store a [`Bundle`] of components with a matching archetype in this storage.
    /// # Returns
    ///     - `None`: If the archetypes aren't matching, or one of the components wasn't registered.
    ///     - `Some(usize)`: The index (a.k.a row) where the components of the bundle are stored.
    pub fn store_bundle<B: Bundle + Archetype>(
        &mut self,
        comp_factory: &ComponentFactory,
        bundle: B,
    ) -> Option<usize> {
        B::arch_info(comp_factory)?
            .prime_key()
            .is_matching_archetype(self.prime_key)
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
    ) -> usize {
        bundle.raw_components_scope(comp_factory, &mut |comp_id, raw_comp| {
            self.store_component_unchecked(comp_id, raw_comp)
        });
        self.len += 1;
        self.len
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

    /// Query this storage for a bundle of components.
    pub fn query<A: Archetype>(&self, index: usize) -> Option<A> {
        todo!()
    }

    /// Get a type-erased reference to a pointer, from its index and [`ComponentId`].
    pub fn get_component_by_id(&self, index: usize, comp_id: ComponentId) -> Option<Ptr<'_>> {
        todo!()
    }

    /// Get a reference to a single component, from its index.
    pub fn get_component<C: Component>(&self, index: usize) -> Option<&C> {
        todo!()
    }
}
