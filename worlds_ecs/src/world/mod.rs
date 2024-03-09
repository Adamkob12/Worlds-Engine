use crate::{
    archetype::Archetype,
    entity::{EntityId, EntityMeta},
    prelude::{ArchFilter, ArchQuery, Bundle, Component},
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
//                               QUERIES API
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

impl World {
    /// Query the world for components.
    // TODO: Better docs + examples
    pub fn query<Q: ArchQuery>(&mut self) -> impl Iterator<Item = Q::Item<'_>> + '_ {
        // SAFETY: The query is safe to use, because the pointer to the storages came from a &mut.
        unsafe { Q::iter_query_matches(&mut self.storages.arch_storages, &self.components) }
    }

    /// Query the world for components, with a filter.
    // TODO: Better docs + examples
    pub fn query_filtered<Q: ArchQuery, F: ArchFilter>(
        &mut self,
    ) -> impl Iterator<Item = Q::Item<'_>> + '_ {
        // SAFETY: The query is safe to use, because the pointer to the storages came from a &mut.
        unsafe {
            Q::iter_filtered_query_matches::<F>(&mut self.storages.arch_storages, &self.components)
        }
    }
}

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
        let index = storage.next_index();
        let entity_id = self.entities.new_entity(EntityMeta {
            archetype_storage_id: sid,
            archetype_storage_index: index,
        });
        storage.store_entity(entity_id, bundle, &self.components);
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
        let entity_meta = self
            .entities
            .get_entity_meta(entity)
            .expect("Can't despawn already despawned entity.");
        if let Some(entity_to_update) = self
            .storages
            .arch_storages
            .get_storage_mut(entity_meta.archetype_storage_id)
            .unwrap()
            .swap_remove(entity_meta.archetype_storage_index)
        {
            self.entities.set_entity_arch_storage_index(
                entity_meta.archetype_storage_index,
                entity_to_update,
            );
        }
        self.entities.remove_entity(entity);
    }
}

#[cfg(test)]
mod tests {
    use crate::{entity::EntityId, prelude::*, world::storage::storages::ArchStorageId};

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

    #[test]
    fn test_despawning_entities_1() {
        let mut world = World::default();

        let a_cart = world.spawn((A(1), C(String::from("Cart"))));
        let a_alice = world.spawn((A(2), C(String::from("Alice"))));
        let a_james = world.spawn((A(3), C(String::from("James"))));

        assert_eq!(
            world
                .storages
                .arch_storages
                .get_storage(ArchStorageId(0))
                .unwrap()
                .len(),
            3
        );
        assert_eq!(world.query::<(&A, &C)>().into_iter().count(), 3);

        world.despawn(a_cart);
        assert_eq!(world.get_component::<A>(a_alice).unwrap().0, 2);
        assert_eq!(world.get_component::<A>(a_james).unwrap().0, 3);
        assert!(world.get_component::<A>(a_cart).is_none());

        assert_eq!(
            world
                .storages
                .arch_storages
                .get_storage(ArchStorageId(0))
                .unwrap()
                .len(),
            2
        );
        world
            .query_filtered::<EntityId, Has<(A, C)>>()
            .into_iter()
            .for_each(|eid| assert_ne!(eid, a_cart));
        assert_eq!(world.query::<(&A, &C)>().into_iter().count(), 2);
    }

    #[test]
    fn test_despawning_entities_2() {
        let mut world = World::default();
        let mut entities = Vec::new();

        (0..1000).for_each(|i| entities.push(world.spawn(A(i))));

        (0..1000)
            .filter(|i| i % 2 == 0)
            .for_each(|i| world.despawn(entities[i]));

        assert_eq!(world.query::<&A>().into_iter().count(), 500);
        world.query::<&A>().for_each(|A(i)| assert!(i % 2 == 1));
    }
}
