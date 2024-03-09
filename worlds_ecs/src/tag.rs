use crate::utils::TypeIdMap;
use std::{any::TypeId, sync::Arc};

/// A tag is a marker that can be added and removed from entities. It contains no data.
pub trait Tag: 'static {}

/// A data-strucutre that can be used to create and manage tags.
pub struct TagFactory {
    tag_id_map: TypeIdMap<u32>,
    next_id: u32,
}

/// Tracks which tags are present on an entity.
pub struct TagTracker {
    tags: Arc<[bool]>,
    factory: Arc<TagFactory>,
}

impl Clone for TagTracker {
    fn clone(&self) -> Self {
        Self {
            tags: Arc::clone(&self.tags),
            factory: Arc::clone(&self.factory),
        }
    }
}

impl Default for TagFactory {
    fn default() -> Self {
        Self {
            tag_id_map: TypeIdMap::default(),
            next_id: 0,
        }
    }
}

impl TagFactory {
    /// Create a new tag.
    pub fn register_tag<T: Tag>(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.tag_id_map.insert(TypeId::of::<T>(), id);
        id
    }

    /// Get the ID of a tag.
    pub fn tag_id<T: Tag>(&self) -> Option<u32> {
        self.tag_id_map.get(&TypeId::of::<T>()).copied()
    }

    /// Get the ID of a tag, without checking whether it exists.
    pub unsafe fn tag_id_unchecked<T: Tag>(&self) -> u32 {
        *self.tag_id_map.get(&TypeId::of::<T>()).unwrap_unchecked()
    }

    /// Produce a new [`TagTracker`] to track which tags are present on an entity.
    pub fn new_tracker(this: &Arc<TagFactory>) -> TagTracker {
        TagTracker {
            tags: vec![false; this.next_id as usize].into(),
            factory: Arc::clone(this),
        }
    }
}

impl TagTracker {
    /// Set this [`Tag`] as present.
    /// # Safety
    /// The caller must ensure that:
    /// - The tag is registered.
    /// - No other [`TagTracker`]s of the same entity are being accessed.
    pub unsafe fn tag<T: Tag>(&mut self) {
        let id = self.factory.tag_id_unchecked::<T>();
        Arc::get_mut_unchecked(&mut self.tags)[id as usize] = true;
    }

    /// Set this [`Tag`] as not present.
    /// # Safety
    /// The caller must ensure that:
    /// - The tag is registered.
    /// - No other [`TagTracker`]s of the same entity are being accessed.
    pub unsafe fn untag<T: Tag>(&mut self) {
        let id = self.factory.tag_id_unchecked::<T>();
        Arc::get_mut_unchecked(&mut self.tags)[id as usize] = false;
    }

    /// Toggle this [`Tag`]. (If it is present, remove it; if it is not present, add it.)
    /// # Safety
    /// The caller must ensure that:
    /// - The tag is registered.
    /// - No other [`TagTracker`]s of the same entity are being accessed.
    pub unsafe fn toggle_unchecked<T: Tag>(&mut self) {
        let id = self.factory.tag_id_unchecked::<T>();
        let current = self.is_tagged::<T>();
        Arc::get_mut_unchecked(&mut self.tags)[id as usize] = !current;
    }

    /// Check if this [`Tag`] is registered.
    pub fn is_tag_registered<T: Tag>(&self) -> bool {
        self.factory.tag_id::<T>().is_some()
    }

    /// Check if this [`Tag`] is present in this tracker.
    /// # Safety
    /// The caller must ensure that:
    /// - No other [`TagTracker`]s of the same entity are being mutated.
    pub unsafe fn is_tagged<T: Tag>(&self) -> bool {
        let id = self.factory.tag_id::<T>().unwrap();
        self.tags[id as usize]
    }

    /// Check if this [`Tag`] is present in this tracker, without checking whether it exists.
    pub unsafe fn is_tagged_unchecked<T: Tag>(&self) -> bool {
        let id = self.factory.tag_id_unchecked::<T>();
        self.tags[id as usize]
    }

    /// Remove all tags from this tracker.
    /// # Safety
    /// The caller must ensure that:
    /// - No other [`TagTracker`]s of the same entity are being accessed.
    pub unsafe fn untag_all(&mut self) {
        Arc::get_mut_unchecked(&mut self.tags)
            .iter_mut()
            .for_each(|tag| *tag = false);
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[derive(Tag)]
    struct Flying;

    #[derive(Tag)]
    struct HasWings;

    #[derive(Component)]
    struct Bird(&'static str);

    #[derive(Component)]
    struct FlyingSpeed(f32);

    #[test]
    fn test_tags() {
        let mut tagf = TagFactory::default();
        tagf.register_tag::<Flying>();
        tagf.register_tag::<HasWings>();

        let mut world = World::with_tags(tagf);

        let eagle = world.spawn((Bird("Eagle"), FlyingSpeed(10.0)));

        let mut eagle_tracker = world.get_tag_tracker(eagle);

        unsafe {
            eagle_tracker.tag::<Flying>();
            eagle_tracker.tag::<HasWings>();
        }

        unsafe {
            assert!(eagle_tracker.is_tagged::<Flying>());
            assert!(eagle_tracker.is_tagged::<HasWings>());
        }

        unsafe {
            eagle_tracker.untag::<Flying>();
            eagle_tracker.untag_all();
        }

        unsafe {
            assert!(!eagle_tracker.is_tagged::<Flying>());
            assert!(!eagle_tracker.is_tagged::<HasWings>());
        }
    }
}
