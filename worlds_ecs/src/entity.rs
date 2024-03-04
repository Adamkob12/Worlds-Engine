use crate::world::storage::{arch_storage::ArchStorageIndex, storages::ArchStorageId};
use std::collections::VecDeque;

/// A unique identifer for an entity in the in the [`World`](crate::world::World)
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EntityId {
    id: u32,
    gen: u32,
}

impl EntityId {
    fn new(id: u32) -> EntityId {
        EntityId { id, gen: 0 }
    }

    /// The unique Id of this entity.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// The generation of this entity.
    pub fn generation(&self) -> u32 {
        self.gen
    }

    /// With generation
    pub fn with_generation(mut self, gen: u32) -> EntityId {
        self.gen = gen;
        self
    }
}

/// A data structure to keep track of all the entities in the world, and their information.
// TODO: Better docs
#[derive(Default)]
pub struct EntityFactory {
    /// Indexed by an [`EntityId::id`], this list keeps track of the current generation of each entity.
    generations: Vec<u32>,
    /// Queued [`EntityId`]s are ids of entities that have been removed. If the queue is non-empty, the next
    /// entity that this [`EntityFactory`] will produce with have the same id as the [`EntityId`] in the head of this
    /// queue, with a greater generation. If the queue is empty, this [`EntityFactory`] will allocate a new entity with
    /// a new unique [`EntityId`].
    queued_entitys: VecDeque<EntityId>,
    /// Meta-data of entities. Indexed by [`EntityId::id`].
    entity_metas: Vec<EntityMeta>,
    /// Number of registered entities, also the length of [`Self::entity_metas`] & [`Self::generations`].
    entities: u32,
}

impl EntityFactory {
    /// Allocate a new entity, and return its [`EntityId`]. Note this is different from [`Self::new_entity`]
    /// because this will always *allocate* a new entity, whereas [`Self::new_entity`] could also pull from
    /// the depspawned entity queue. Panics if the maximum amount of entities has been reached (2^32).
    fn alloc_new_entity(&mut self, entity_meta: EntityMeta) -> EntityId {
        self.generations.push(0);
        self.entity_metas.push(entity_meta);

        EntityId::new(self.entities - 1)
    }

    /// Produce a new entity, and return its [`EntityId`]. Note this is different from [`Self::alloc_new_entity`]
    /// & [`Self::new_entity`] because this will only use the [`EntityId`] of an entity that was removed.
    /// Panics if the maximum amount of entities has been reached (2^32).
    fn revive_removed_entity(&mut self, entity_meta: EntityMeta) -> Option<EntityId> {
        let id = self.queued_entitys.pop_front()?;
        let entity = id.with_generation(self.generations[id.id() as usize]);
        self.set_entity_meta(entity_meta, entity);
        Some(entity)
    }

    /// Produce a new entity, and return its [`EntityId`]. Note this is different from [`Self::alloc_new_entity`]
    /// because this can create reuse a removed entity's [`EntityId`], whereas [`Self::alloc_new_entity`]
    /// will always allocate a new entity. Panics if the maximum amount of entities has been reached (2^32).
    pub fn new_entity(&mut self, entity_meta: EntityMeta) -> EntityId {
        self.entities += 1;
        self.revive_removed_entity(entity_meta)
            .unwrap_or(self.alloc_new_entity(entity_meta))
    }

    /// Verify the generation of this entity, meaning, verify that it hasn't been removed.
    pub fn verify_generation(&self, entity: EntityId) -> bool {
        self.generations[entity.id() as usize] == entity.gen
    }

    /// remove an entity. This will increment the generation matching this entity's [`id`](EntityId::id).
    /// And add it to the queue of removed entities. Panic if the entity doesn't exist.
    pub fn remove_entity(&mut self, entity: EntityId) {
        assert!(
            self.verify_generation(entity),
            "Can't remove removed entity"
        );
        self.generations[entity.id() as usize] += 1;
        self.entities -= 1;
        self.queued_entitys.push_back(entity)
    }

    /// The the [`EntityMeta`] of an entity.
    pub fn get_entity_meta(&self, entity: EntityId) -> Option<&EntityMeta> {
        self.verify_generation(entity)
            .then(|| &self.entity_metas[entity.id() as usize])
    }

    /// Set the [`EntityMeta`] of an entity.
    pub fn set_entity_meta(&mut self, entity_meta: EntityMeta, entity: EntityId) {
        self.entity_metas[entity.id() as usize] = entity_meta
    }

    /// Returns how many entities are there in the world.
    pub fn entities(&self) -> u32 {
        self.entities
    }
}

/// Meta-data of an entity.
#[derive(Clone, Copy)]
pub struct EntityMeta {
    pub(crate) archetype_storage_id: ArchStorageId,
    pub(crate) archetype_storage_index: ArchStorageIndex,
}

impl EntityMeta {
    #[allow(unused)]
    pub(crate) const PLACEHOLDER: EntityMeta = EntityMeta {
        archetype_storage_id: ArchStorageId(usize::MAX),
        archetype_storage_index: ArchStorageIndex(usize::MAX),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entities() {
        let mut entity_factory = EntityFactory::default();
        let mut entities = Vec::new();
        (0..100).for_each(|_| {
            entities.push(entity_factory.new_entity(EntityMeta::PLACEHOLDER));
        });

        for entity in &entities {
            assert!(entity_factory.verify_generation(*entity));
        }

        for entity in &entities {
            assert!(entity_factory.get_entity_meta(*entity).is_some());
        }

        for entity in entities.iter().filter(|id| id.id() % 2 == 0) {
            entity_factory.remove_entity(*entity);
        }

        for entity in &entities {
            assert!(entity_factory.verify_generation(*entity) || entity.id() % 2 == 0);
        }

        assert_eq!(entity_factory.entities(), 50);

        (0..50).for_each(|_| {
            entity_factory.new_entity(EntityMeta::PLACEHOLDER);
        });

        assert_eq!(entity_factory.entities(), 100);
    }
}
