/// Module responsible for any data that can be stored in the World.
pub mod data;
/// Module responsible for storage in the World.
pub mod storage;

/// This type stores everything that is offered by this crate. It is the main type of the ECS.
/// It exposes the API for the ECS, it is the bedrock of the engine.
// TODO: Better docs
pub struct World {
    _data: data::WorldData,
    _components: crate::component::ComponentFactory,
    _entities: crate::entity::EntityFactory,
    _storages: storage::storages::StorageFactory,
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//                               MISC. API
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

impl World {}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//                               COMPONENTS API
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

impl World {}
