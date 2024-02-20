#![deny(missing_docs)]
//! The ECS for the Red Flower Game Engine

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
    pub use super::component;
    pub use super::storage;
}
