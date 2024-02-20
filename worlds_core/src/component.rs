use std::any::TypeId;

use crate::{
    impl_id_struct,
    utils::{
        prime_key::{PrimeArchKey, MAX_COMPONENTS},
        TypeIdMap,
    },
    world::data::{Data, DataInfo},
};

/// The trait that represents a component.
pub trait Component: Data {}

/// A unique identifer for a [`Component`] in the [`World`](crate::world::World)
#[derive(Copy, Clone)]
pub struct ComponentId(usize);
impl_id_struct!(ComponentId);

impl ComponentId {
    pub(crate) fn _prime_key(&self) -> PrimeArchKey {
        PrimeArchKey::component_key(*self)
    }
}

/// A data structure to keep track of all the components in the world, and their information.
#[derive(Default)]
pub struct Components {
    /// Map the [`TypeId`] of each [`Component`] to its [`ComponentId`]
    type_map: TypeIdMap<ComponentId>,
    /// The [`DataInfo`] for each component, indexed by [`ComponentId`]
    components: Vec<DataInfo>,
}

impl Components {
    /// Register a new component from a generic type.
    /// If this component is already registered, this method will return
    /// the [`ComponentId`] of the previously registered component.
    /// If the component couldn't be registered for some reason, return `None`
    /// (the reason is most likely that the maximum amount of registered components has been reached.)
    pub fn register_component<C: Component>(&mut self) -> Option<ComponentId> {
        self.register_component_from_data(TypeId::of::<C>(), DataInfo::deafult_for::<C>())
    }

    /// Register a new component from raw data.
    /// If a component with this [`TypeId`] exists already, this method will return
    /// the [`ComponentId`] of the previously registered component.
    /// If the component couldn't be registered for some reason, return `None`
    /// (the reason is most likely that the maximum amount of registered components has been reached.)
    pub fn register_component_from_data(
        &mut self,
        type_id: TypeId,
        data_info: DataInfo,
    ) -> Option<ComponentId> {
        if self.is_type_registered(type_id) {
            return self.get_component_id_from_type_id(type_id);
        }
        (self.components.len() < MAX_COMPONENTS)
            .then_some(self.register_component_from_data_unchecked(type_id, data_info))
    }

    /// Register a new component like [`Self::register_component_from_data`] without checking whether this
    /// component is already registered, and whether the [`maximum amount of components`](MAX_COMPONENTS) has been reached.
    /// This method is not unsafe, but using it without caution may result in difficult to find bugs and / or wasted memory.
    pub fn register_component_from_data_unchecked(
        &mut self,
        type_id: TypeId,
        data_info: DataInfo,
    ) -> ComponentId {
        let comp_id = ComponentId::new(self.components.len());
        self.type_map.insert(type_id, comp_id);
        self.components.push(data_info);
        comp_id
    }

    /// Register a new component like [`Self::register_component`] without checking whether this
    /// component is already registered, and whether the [`maximum amount of components`](MAX_COMPONENTS) has been reached.
    /// This method is not unsafe, but using it without caution may result in difficult to find bugs and / or wasted memory.
    pub fn register_component_unchecked<C: Component>(&mut self) -> ComponentId {
        self.register_component_from_data_unchecked(TypeId::of::<C>(), DataInfo::deafult_for::<C>())
    }

    /// Get the [`DataInfo`] of a component
    pub fn get_component_info<C: Component>(&self) -> Option<&DataInfo> {
        self.get_component_info_from_type_id(TypeId::of::<C>())
    }

    /// Get the [`DataInfo`] of a component from its [`TypeId`]
    pub fn get_component_info_from_type_id(&self, type_id: TypeId) -> Option<&DataInfo> {
        self.type_map.get(&type_id).map(|id| {
            self.get_component_info_from_component_id(*id)
                .expect("ComponentId stored internally was wrong")
        })
    }

    /// Get the [`DataInfo`] of a component from its [`ComponentId`]
    pub fn get_component_info_from_component_id(&self, comp_id: ComponentId) -> Option<&DataInfo> {
        self.components.get(comp_id.id())
    }

    /// Get the [`ComponentId`] of a component
    pub fn get_component_id<C: Component>(&self) -> Option<ComponentId> {
        self.get_component_id_from_type_id(TypeId::of::<C>())
    }

    /// Get the [`ComponentId`] of a component from it's [`TypeId`]
    pub fn get_component_id_from_type_id(&self, type_id: TypeId) -> Option<ComponentId> {
        self.type_map.get(&type_id).copied()
    }

    /// Returns `true` if the component is registered. `false` if not.
    pub fn is_registered<C: Component>(&self) -> bool {
        self.type_map.contains_key(&TypeId::of::<C>())
    }

    /// Returns `true` if a component with this [`TypeId`] is registered. `false` if not.
    pub fn is_type_registered(&self, type_id: TypeId) -> bool {
        self.type_map.contains_key(&type_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct A;
    struct B;
    struct C;

    impl Data for A {}
    impl Component for A {}

    impl Data for B {}
    impl Component for B {}

    impl Data for C {}
    impl Component for C {}

    #[test]
    fn test_components() {
        let mut components = Components::default();

        assert!(!components.is_registered::<A>());
        assert!(!components.is_registered::<B>());
        assert!(!components.is_registered::<C>());

        components.register_component::<A>();
        components.register_component::<B>();
        components.register_component_unchecked::<C>();

        assert!(components.is_registered::<A>());
        assert!(components.is_registered::<B>());
        assert!(components.is_registered::<C>());

        assert_eq!(components.get_component_id::<A>().unwrap().id(), 0);
        assert_eq!(components.get_component_id::<B>().unwrap().id(), 1);
        assert_eq!(components.get_component_id::<C>().unwrap().id(), 2);
        assert_eq!(
            components.get_component_info::<A>().unwrap().layout(),
            components.get_component_info::<B>().unwrap().layout()
        );
        assert_ne!(
            components.get_component_info::<A>().unwrap().name(),
            components.get_component_info::<B>().unwrap().name()
        );
        assert_eq!(
            components.get_component_info::<A>().unwrap().name(),
            "worlds_core::component::tests::A"
        );
        assert_eq!(
            components.get_component_info::<B>().unwrap().name(),
            "worlds_core::component::tests::B"
        );
        assert_eq!(
            components.get_component_info::<C>().unwrap().name(),
            "worlds_core::component::tests::C"
        );
    }
}