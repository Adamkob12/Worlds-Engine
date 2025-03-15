#[allow(unused_imports)] // For the docs
use crate::world::World;
use bevy_ptr::OwningPtr;
use std::{alloc::Layout, any::type_name};

/// Piece of Data in the [`World`]
pub trait Data: 'static + Send + Sync {}

#[allow(unused)]
/// Information for a data. Some of it is critical for storage, such as the memory [`Layout`], some is less important, like the name.
pub struct DataInfo {
    /// The name of the [`Data`].
    name: &'static str,
    /// The memory layout of the [`Data`].
    layout: Layout,
    /// If there should be some kind of action taken when the data is dropped,
    /// it is represented in this function. The function takes an [`OwningPtr`] to this data, which is
    /// guarenteed to match the data's type.
    drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
}

unsafe fn drop_data<T: Data>(ptr: OwningPtr<'_>) { unsafe {
    OwningPtr::drop_as::<T>(ptr)
}}

impl DataInfo {
    /// Create a new [`DataInfo`] for a value based on its default values.
    pub fn deafult_for<T: Data>() -> Self {
        Self {
            name: type_name::<T>(),
            layout: Layout::new::<T>(),
            drop_fn: Some(drop_data::<T>),
        }
    }

    /// Get this [`Data`]'s type-erased drop function
    pub fn drop_fn(&self) -> Option<unsafe fn(OwningPtr<'_>)> {
        self.drop_fn
    }

    /// Get this [`Data`]'s memory layout
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// Get this [`Data`]'s name
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Create a raw and unchekced [`DataInfo`].
    pub fn new(
        name: &'static str,
        layout: Layout,
        drop_fn: Option<unsafe fn(OwningPtr<'_>)>,
    ) -> DataInfo {
        Self {
            layout,
            drop_fn,
            name,
        }
    }
}
