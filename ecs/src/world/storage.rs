use std::collections::HashMap;

use crate::{
    archetype::{Archetype, MAX_COMPS_PER_ARCH},
    prelude::{ComponentFactory, ComponentId},
    storage::blob_vec::BlobVec,
    utils::prime_key::PrimeArchKey,
};

/// A data-structure that stores the data of an archetype. Specifically, it's components.
#[allow(unused)]
pub struct ArchStorage {
    /// By indexing this list using [`ComponentId::id`], we get the index to the component's storage
    /// in the `comp_storage` field.
    comp_indexes: HashMap<ComponentId, usize>,
    /// The raw storage of the components.
    comp_storage: Box<[BlobVec]>,
    /// The [`PrimeArchKey`] of the archetype stored here.
    prime_key: PrimeArchKey,
}

impl ArchStorage {
    /// Create a new [`ArchStorage`] for an archetype
    pub fn new<A: Archetype>(comp_factory: &ComponentFactory) -> Option<ArchStorage> {
        let arch_info = A::arch_info(comp_factory)?;
        let components = arch_info.components();
        let mut comp_storage = Vec::new();
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
            comp_storage: comp_storage.into_boxed_slice(),
        })
    }
}
