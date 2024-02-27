#![deny(missing_docs)]
//! The ECS for the Red Flower Game Engine

/// Module responsible for anything to do archetypes.
pub mod archetype;
/// Module responsible for anything to do with bundles.
pub mod bundle;
/// Module responsible for anything to do with components.
pub mod component;
/// Module responsible for anything to do with entities.
pub mod entity;
/// Module responsible for anything to do with storage.
pub mod storage;
/// Module responsible for anything to do with the world.
pub mod world;

pub(crate) mod utils;

/// The common and useful exports of this crate.
pub mod prelude {
    pub use super::bundle::Bundle;
    pub use super::component;
    pub use super::component::*;
    pub use super::storage;
    pub use super::world::data::{Data, DataInfo, WorldData};
    pub use super::world::World;
    pub use worlds_derive::Component;
}
