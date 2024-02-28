use super::arch_storage::ArchStorage;

/// A data structure to keep track of all the storages in the world, and their information.
pub struct StorageFactory {
    /// All the [`ArchStorage`]s in the [`World`](crate::prelude::World)
    archetype_storages: Vec<ArchStorage>,
}
