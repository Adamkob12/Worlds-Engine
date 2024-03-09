#![deny(missing_docs)]
#![feature(get_mut_unchecked)]
//! The ECS for the Worlds Engine.

/// Module responsible for anything to do archetypes.
pub mod archetype;
/// Module responsible for anything to do with bundles.
pub mod bundle;
/// Module responsible for anything to do with components.
pub mod component;
/// Module responsible for anything to do with entities.
pub mod entity;
/// Module responsible for anything to do with queries.
pub mod query;
/// Module responsible for anything to do with storage.
pub mod storage;
/// Module responsible for anything to do with tags.
pub mod tag;
/// Module responsible for anything to do with the world.
pub mod world;

pub(crate) mod utils;

/// The common and useful exports of this crate.
pub mod prelude {
    pub use super::bundle::Bundle;
    pub use super::component;
    pub use super::component::*;
    pub use super::query::*;
    pub use super::storage;
    pub use super::tag::*;
    pub use super::world::data::*;
    pub use super::world::World;
    pub use worlds_derive::{Component, Tag};
}
