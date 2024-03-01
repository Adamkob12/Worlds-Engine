use crate::{archetype::Archetype, prelude::ComponentFactory, utils::prime_key::PrimeArchKey};

use super::arch_storage::ArchStorage;

/// A data structure to keep track of all the storages in the world, and their information.
// TODO: Better docs
pub struct StorageFactory {
    pub(crate) arch_storages: ArchStorages,
}

/// All the [`ArchStorage`]s in the [`World`](crate::prelude::World)
pub struct ArchStorages {
    storages: Vec<ArchStorage>,
    pkeys: Vec<PrimeArchKey>,
}

/// Identifies an [`ArchStorage`] in the [`StorageFactory`]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ArchStorageId(usize);

impl ArchStorages {
    /// Get a shared reference to an [`ArchStorage`] from its [`ArchStorageId`]
    pub fn get_storage(&self, id: ArchStorageId) -> Option<&ArchStorage> {
        self.storages.get(id.0)
    }

    /// Get an exclusive reference to an [`ArchStorage`] from its [`ArchStorageId`]
    pub fn get_storage_mut(&mut self, id: ArchStorageId) -> Option<&mut ArchStorage> {
        self.storages.get_mut(id.0)
    }

    /// Get a shared reference to an [`ArchStorage`] from its [`ArchStorageId`], without doing any bounds checking
    pub unsafe fn get_storage_unchecked(&self, id: ArchStorageId) -> &ArchStorage {
        self.storages.get_unchecked(id.0)
    }

    /// Get an exclusive reference to an [`ArchStorage`] from its [`ArchStorageId`], without doing any bounds checking
    pub unsafe fn get_storage_mut_unchecked(&mut self, id: ArchStorageId) -> &mut ArchStorage {
        self.storages.get_unchecked_mut(id.0)
    }

    /// Get the [`ArchStorage`]s that stores archetypes with the exact same [`PrimeArchKey`]
    pub fn get_storage_with_exact_archetype(&self, pkey: PrimeArchKey) -> Option<&ArchStorage> {
        self.pkeys
            .iter()
            .zip(&self.storages)
            .find_map(move |(p, storage)| p.is_exact_archetype(pkey).then_some(storage))
    }

    /// Get mutable access to the [`ArchStorage`]s that stores archetypes with the exact same [`PrimeArchKey`]
    pub fn get_mut_storage_with_exact_archetype(
        &mut self,
        pkey: PrimeArchKey,
    ) -> Option<&mut ArchStorage> {
        self.pkeys
            .iter_mut()
            .zip(&mut self.storages)
            .find_map(move |(p, storage)| p.is_exact_archetype(pkey).then_some(storage))
    }

    /// Iterate over all of the [`ArchStorage`]s that store archetypes with a matching archetype of `pkey`.
    /// Meaning the table's archetype is a sub-archetype of the archetype represented by `pkey`. For example:
    /// For components: A, B, C, D, E
    /// For archetypes storages (represented by the archetypes they store): (A, B, C, D, E), (A, B), (D), (D, E)
    /// The archetypes storages "matching" the archetype (D, E) are: (A, B, C, D, E) and (D, E)
    pub fn iter_storages_with_matching_archetype(
        &self,
        pkey: PrimeArchKey,
    ) -> impl Iterator<Item = &ArchStorage> + '_ {
        self.pkeys
            .iter()
            .zip(&self.storages)
            .filter_map(move |(p, storage)| p.is_sub_archetype(pkey).then_some(storage))
    }

    /// Iterate over all of the [`ArchStorage`]s that store archetypes with a matching archetype of `pkey` mutably.
    /// Meaning the table's archetype is a sub-archetype of the archetype represented by `pkey`. For example:
    /// For components: A, B, C, D, E
    /// For archetypes storages (represented by the archetypes they store): (A, B, C, D, E), (A, B), (D), (D, E)
    /// The archetypes storages "matching" the archetype (D, E) are: (A, B, C, D, E) and (D, E)
    pub fn iter_storages_with_matching_archetype_mut(
        &mut self,
        pkey: PrimeArchKey,
    ) -> impl Iterator<Item = &mut ArchStorage> + '_ {
        self.pkeys
            .iter_mut()
            .zip(&mut self.storages)
            .filter_map(move |(p, storage)| p.is_sub_archetype(pkey).then_some(storage))
    }

    /// Checks if this archetype is stored here.
    pub fn is_archetype_stored<A: Archetype>(&self, comp_factory: &ComponentFactory) -> bool {
        A::prime_key(comp_factory).map_or(false, |pkey1| {
            self.pkeys
                .iter()
                .find(|pkey2| pkey2.is_exact_archetype(pkey1))
                .map_or(false, |_| true)
        })
    }

    /// Internally, create a new [`ArchStorage`] to store the given archetype. Returns `None` if there was
    /// already an [`ArchStorage`] storing the given archetype. If there were no previous storages storing the
    /// given [`Archetype`], a new one is created an its [`PrimeArchKey`] is returned.
    pub fn store_new_archetype_checked<A: Archetype>(
        &mut self,
        comp_factory: &ComponentFactory,
    ) -> Option<ArchStorageId> {
        (A::arch_info(comp_factory).is_some() && self.is_archetype_stored::<A>(comp_factory))
            // SAFETY: We checked that the components are registered, and that archetype isn't being stored already.
            .then_some(unsafe { self.store_new_archetype_unchecked::<A>(comp_factory) })
    }

    /// Internally, create a new [`ArchStorage`] to store the given archetype. Without checking if a previous
    /// [`ArchStorage`] already exists for this [`Archetype`], or if the components are registered in the [`ComponentFactory`].
    /// # Safety
    /// The caller must ensure that:
    ///     - All of the components in the [`Archetype`] are registered in the [`ComponentFactory`].
    ///     - The archetype isn't currently being stored in [`Self`] (using the [`Self::is_archetype_stored`] method.)
    pub unsafe fn store_new_archetype_unchecked<A: Archetype>(
        &mut self,
        comp_factory: &ComponentFactory,
    ) -> ArchStorageId {
        self.storages
            .push(ArchStorage::new::<A>(comp_factory).unwrap_unchecked());
        let pkey = A::prime_key(comp_factory).unwrap_unchecked();
        self.pkeys.push(pkey);
        ArchStorageId(self.pkeys.len() - 1)
    }
}
