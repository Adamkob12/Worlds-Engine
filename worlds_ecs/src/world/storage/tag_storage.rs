use std::sync::Arc;

use crate::{
    entity::EntityId,
    tag::{TagFactory, TagTracker},
};

/// A data-structure to keep track of which entities have which tags.
pub struct TagStorage {
    /// The [`TagTracker`] for each entity, indexed by the entity's id.
    tag_trackers: Vec<TagTracker>,
    /// The factory to create and manage tags.
    tag_factory: Arc<TagFactory>,
}

impl Default for TagStorage {
    fn default() -> Self {
        Self {
            tag_trackers: Vec::new(),
            tag_factory: Arc::new(TagFactory::default()),
        }
    }
}

impl TagStorage {
    /// Create a new [`TagStorage`] with the given [`TagFactory`].
    pub fn new(tagf: Arc<TagFactory>) -> Self {
        Self {
            tag_trackers: Vec::new(),
            tag_factory: Arc::clone(&tagf),
        }
    }

    /// Creates room to store the [`TagTracker`] of a new entity.
    pub fn new_entity(&mut self) {
        self.tag_trackers
            .push(TagFactory::new_tracker(&self.tag_factory));
    }

    /// Untag all of the tags of an entity.
    pub fn untag_all(&mut self, entity: EntityId) {
        // SAFETY: No other `TagTracker`s are being accessed
        unsafe { self.tag_trackers[entity.id() as usize].untag_all() }
    }

    /// Get the [`TagTracker`] of an entity.
    pub fn get_tag_tracker(&self, entity: EntityId) -> TagTracker {
        self.tag_trackers[entity.id() as usize].clone()
    }

    /// Get the [`TagTracker`] of an entity, without checking if the entity exists.
    pub unsafe fn get_tag_tracker_unchecked(&self, entity: EntityId) -> TagTracker { unsafe {
        self.tag_trackers
            .get_unchecked(entity.id() as usize)
            .clone()
    }}
}
