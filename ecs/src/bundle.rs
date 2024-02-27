use bevy_ptr::OwningPtr;

use crate::{
    archetype::Archetype,
    prelude::{Component, ComponentFactory, ComponentId},
};

pub(crate) trait Bundle: Archetype {
    fn register_components(&self, comp_factory: &mut ComponentFactory);

    fn fetch_components(self, comp_factory: &ComponentFactory) -> Self;
}

impl<C> Bundle for C
where
    C: Component,
{
    fn register_components(&self, comp_factory: &mut ComponentFactory) {
        comp_factory
            .register_component::<C>()
            .expect("Couldn't register component.");
    }

    fn fetch_components(self, comp_factory: &ComponentFactory) -> Self {
        let comp_id = comp_factory
            .get_component_id::<C>()
            .expect("Can't fetch unregistered component from bundle. It must be initialized.");
    }
}
