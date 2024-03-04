use crate::{
    archetype::Archetype,
    entity::{EntityId, EntityMeta},
    prelude::{Bundle, Component},
};

/// Module responsible for any data that can be stored in the World.
pub mod data;
/// Module responsible for storage in the World.
pub mod storage;

/// This type stores everything that is offered by this crate. It is the main type of the ECS.
/// It exposes the API for the ECS, it is the bedrock of the engine.
// TODO: Better docs
#[derive(Default)]
pub struct World {
    pub(crate) _data: data::WorldData,
    pub(crate) components: crate::component::ComponentFactory,
    pub(crate) entities: crate::entity::EntityFactory,
    pub(crate) storages: storage::storages::StorageFactory,
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
    pub fn spawn<B: Bundle + Archetype>(&mut self, bundle: B) -> EntityId {
        let (sid, storage) = self
            .storages
            .arch_storages
            .get_mut_or_create_storage_with_exact_archetype::<B>(&mut self.components);
        let index = storage
            .store_bundle(&self.components, bundle)
            .expect("Coudln't store bundle");
        let entity_id = self.entities.new_entity(EntityMeta {
            archetype_storage_id: sid,
            archetype_storage_index: index,
        });
        entity_id
    }

    /// Get a reference to a [`Component`] of an entity.
    pub fn get_component<C: Component>(&self, entity: EntityId) -> Option<&C> {
        let entity_meta = self.entities.get_entity_meta(entity)?;
        self.storages
            .arch_storages
            .get_storage(entity_meta.archetype_storage_id)
            .map(|storage| {
                self.components
                    .get_component_id::<C>()
                    .map(|comp_id| {
                        storage.get_component(entity_meta.archetype_storage_index, comp_id)
                    })
                    .flatten()
                    // SAFETY: This type-erased pointer was fetched using this component id.
                    .map(|raw_comp| unsafe { raw_comp.deref::<C>() })
            })
            .flatten()
    }

    /// Get a mutable reference to a [`Component`] of an entity.
    pub fn get_component_mut<C: Component>(&mut self, entity: EntityId) -> Option<&mut C> {
        let entity_meta = self.entities.get_entity_meta(entity)?;
        self.storages
            .arch_storages
            .get_storage_mut(entity_meta.archetype_storage_id)
            .map(|storage| {
                self.components
                    .get_component_id::<C>()
                    .map(|comp_id| {
                        storage.get_component_mut(entity_meta.archetype_storage_index, comp_id)
                    })
                    .flatten()
                    // SAFETY: This type-erased pointer was fetched using this component id.
                    .map(|raw_comp| unsafe { raw_comp.deref_mut::<C>() })
            })
            .flatten()
    }

    /// Despawn an entity from the [`World`].
    pub fn despawn(&mut self, entity: EntityId) {
        let _entity_meta = self
            .entities
            .get_entity_meta(entity)
            .expect("Can't despawn already despawned entity.");
        self.entities.remove_entity(entity);
        todo!() // Also remove from storage
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[derive(Component)]
    struct A(usize);

    #[derive(Component)]
    struct B(Box<[u8]>);

    #[derive(Component)]
    struct C(String);

    #[test]
    fn test_world_entities_1() {
        let mut world = World::default();

        let carter = world.spawn((A(1), B(Box::new([10, 20, 30, 40])), C("Carter".into())));
        let alice = world.spawn((A(2), B(Box::new([133, 107])), C("Alice".into())));
        let adam = world.spawn((A(3), B(Box::new([])), C("Adam".into())));

        assert_eq!(world.get_component::<A>(carter).unwrap().0, 1);
        assert_eq!(world.get_component::<A>(alice).unwrap().0, 2);
        assert_eq!(world.get_component::<A>(adam).unwrap().0, 3);

        world.get_component_mut::<A>(carter).unwrap().0 *= 10;
        world.get_component_mut::<A>(alice).unwrap().0 *= 10;
        world.get_component_mut::<A>(adam).unwrap().0 *= 10;

        assert_eq!(world.get_component::<A>(carter).unwrap().0, 10);
        assert_eq!(world.get_component::<A>(alice).unwrap().0, 20);
        assert_eq!(world.get_component::<A>(adam).unwrap().0, 30);

        assert_eq!(world.get_component::<B>(carter).unwrap().0.len(), 4);
        assert_eq!(world.get_component::<B>(alice).unwrap().0.len(), 2);
        assert_eq!(world.get_component::<B>(adam).unwrap().0.len(), 0);

        assert_eq!(&world.get_component::<C>(carter).unwrap().0, "Carter");
        assert_eq!(&world.get_component::<C>(alice).unwrap().0, "Alice");
        assert_eq!(&world.get_component::<C>(adam).unwrap().0, "Adam");
    }

    #[test]
    #[should_panic]
    fn test_multiple_components_1() {
        let mut world = World::default();
        world.spawn((A(0), A(1)));
    }

    #[test]
    #[should_panic]
    fn test_multiple_components_2() {
        let mut world = World::default();
        world.spawn((A(0), A(1), B(Box::new([0, 1]))));
    }
}
