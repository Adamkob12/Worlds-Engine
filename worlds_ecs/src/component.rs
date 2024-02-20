use crate::{impl_id_struct, utils::prime_key::PrimeArchKey, world::data::Data};

/// The trait that represents a component.
pub trait Component: Data {}

/// A unique identifer for a [`Component`] in the [`World`](crate::world::World)
#[derive(Copy, Clone)]
pub struct ComponentId(usize);
impl_id_struct!(ComponentId);

impl ComponentId {
    pub(crate) fn prime_key(&self) -> PrimeArchKey {
        PrimeArchKey::component_key(*self)
    }
}

/// A data structure to keep track of all the components in the world, and their information.
pub struct Components {}
