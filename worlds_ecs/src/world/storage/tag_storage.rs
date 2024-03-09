use std::sync::Arc;

use crate::{
    entity::EntityId,
    tag::{TagFactory, TagTracker},
};

/// A data-structure to keep track of which entities have which tags.
pub struct TagStorage {
    /// The [`TagTracker`] for each entity, indexed by the entity's id.
    tags: Vec<TagTracker>,
    /// The factory to create and manage tags.
    tag_factory: Arc<TagFactory>,
}

impl Default for TagStorage {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            tag_factory: Arc::new(TagFactory::default()),
        }
    }
}

impl TagStorage {
    /// Create a new [`TagStorage`] with the given [`TagFactory`].
    pub fn new(tagf: Arc<TagFactory>) -> Self {
        Self {
            tags: Vec::new(),
            tag_factory: Arc::clone(&tagf),
        }
    }

    /// Creates room to store the [`TagTracker`] of a new entity.
    pub fn new_entity(&mut self) {
        self.tags.push(TagFactory::new_tracker(&self.tag_factory));
    }

    /// Untag all of the tags of an entity.
    pub fn untag_all(&mut self, entity: EntityId) {
        self.tags[entity.id() as usize].untag_all();
    }

    /// Get the [`TagTracker`] of an entity.
    pub fn get_tag_tracker(&self, entity: EntityId) -> TagTracker {
        self.tags[entity.id() as usize]
    }

    /// Get the [`TagTracker`] of an entity, without checking if the entity exists.
    pub unsafe fn get_tag_tracker_unchecked(&self, entity: EntityId) -> TagTracker {
        *self.tags.get_unchecked(entity.id() as usize)
    }
}
