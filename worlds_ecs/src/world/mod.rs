use crate::{
    archetype::Archetype,
    entity::EntityId,
    prelude::{Bundle, Component},
};

/// Module responsible for any data that can be stored in the World.
pub mod data;
/// Module responsible for storage in the World.
pub mod storage;

/// This type stores everything that is offered by this crate. It is the main type of the ECS.
/// It exposes the API for the ECS, it is the bedrock of the engine.
// TODO: Better docs
pub struct World {
    pub(crate) _data: data::WorldData,
    pub(crate) _components: crate::component::ComponentFactory,
    pub(crate) _entities: crate::entity::EntityFactory,
    pub(crate) _storages: storage::storages::StorageFactory,
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//                               MISC. API
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

impl World {}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//                               COMPONENTS API
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

impl World {}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//                               ENTITIES API
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

impl World {
    /// Spawn a new entity with a bundle of components.
    pub fn spawn<B: Bundle + Archetype>(&mut self) -> EntityId {
        todo!()
    }

    /// Get a reference to a [`Component`] of an entity.
    pub fn get_component<C: Component>(&self, entity: EntityId) -> Option<&C> {
        todo!()
    }

    /// Get a mutable reference to a [`Component`] of an entity.
    pub fn get_component_mut<C: Component>(&mut self, entity: EntityId) -> Option<&mut C> {
        todo!()
    }

    /// Despawn an entity from the [`World`].
    pub fn despawn(&mut self, entity: EntityId) {
        todo!()
    }
}
