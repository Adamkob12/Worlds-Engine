use self::arch_storage::{ArchStorage, ArchStorageIndex};
use crate::{
    archetype::Archetype,
    entity::EntityId,
    prelude::{Bundle, ComponentFactory, ComponentId},
};
use bevy_ptr::PtrMut;
use std::ops::Deref;

/// Defining a data-structures to store a bundle of components, a.k.a archetype storage.
pub mod arch_storage;
/// A module to define abstractions around all the storages in the world.
pub mod storages;

/// The storage for entities with the same [`Archetype`]. This holds the actual data of the entities,
/// as well as data about the entities themselves.
pub struct ArchEntityStorage {
    /// The [`ArchStorage`] that stores the actual data.
    arch_storage: ArchStorage,
    /// The Id of each entity in the storage. Indexed by the entity's index in the [`ArchStorage`] ([`ArchStorageIndex`])
    entities: Vec<EntityId>,
}

impl Deref for ArchEntityStorage {
    type Target = ArchStorage;

    fn deref(&self) -> &Self::Target {
        &self.arch_storage
    }
}

impl ArchEntityStorage {
    /// Create a new [`ArchEntityStorage`] for the given [`Archetype`].
    pub fn new<A: Archetype>(compf: &ComponentFactory) -> Option<Self> {
        Some(Self {
            arch_storage: ArchStorage::new::<A>(compf)?,
            entities: Vec::new(),
        })
    }

    /// Get the next index. As in, if a new entity were to be stored right now, that index it would get.
    pub fn next_index(&self) -> ArchStorageIndex {
        ArchStorageIndex(self.len())
    }

    /// Store an entity in the storage, with a [`Bundle`] of components, and return its index.
    pub fn store_entity<B: Bundle + Archetype>(
        &mut self,
        entity_id: EntityId,
        bundle: B,
        compf: &ComponentFactory,
    ) -> Option<ArchStorageIndex> {
        self.entities.push(entity_id);
        self.arch_storage.store_bundle(compf, bundle)
    }

    /// Get a type-erased mutable reference to a pointer, from its index and [`ComponentId`].
    /// Retuns `None` if the index is out of bounds, or if the component is not stored in this storage.
    pub fn get_component_mut(
        &mut self,
        index: ArchStorageIndex,
        comp_id: ComponentId,
    ) -> Option<PtrMut<'_>> {
        self.arch_storage.get_component_mut(index, comp_id)
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
        self.arch_storage
            .get_component_mut_unchecked(index, comp_id)
    }

    /// Get the [`EntityId`] of the entity stored at that index.
    /// Return `None` if the index is out of bounds.
    pub fn get_entity_at(&self, index: ArchStorageIndex) -> Option<EntityId> {
        self.entities.get(index.0).copied()
    }

    /// Get the [`EntityId`] of the entity stored at that index, without doing bounds checking.
    /// # Safety
    /// The caller must ensure that the `index` is valid, and within the bounds of the storage.
    pub unsafe fn get_entity_at_unchecked(&self, index: ArchStorageIndex) -> EntityId {
        *self.entities.get_unchecked(index.0)
    }
}
