use crate::storage::blob_vec::BlobVec;
use crate::utils::TypeIdMap;
#[allow(unused_imports)] // For the docs
use crate::world::World;
use bevy_ptr::OwningPtr;
use std::{
    alloc::Layout,
    any::{type_name, TypeId},
};

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

unsafe fn drop_data<T: Data>(ptr: OwningPtr<'_>) {
    OwningPtr::drop_as::<T>(ptr)
}

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

/// The collection of all the Data in the world.
pub struct WorldData {
    /// Maps a data's [`TypeId`](std::any::TypeId) to its [`DataInfo`]
    id_map: TypeIdMap<DataInfo>,
}

impl WorldData {
    /// Register a new piece of [`Data`] with its default values that can be stored in the [`World`]
    pub fn register_new<T: Data>(&mut self) -> bool {
        self.register_new_with_info::<T>(DataInfo::deafult_for::<T>())
    }

    /// Register a new piece of [`Data`] with provided [`DataInfo`] that can be stored in the [`World`]
    pub fn register_new_with_info<T: Data>(&mut self, data_info: DataInfo) -> bool {
        self.id_map.insert(TypeId::of::<T>(), data_info).is_none()
    }

    /// Generate a type-erased data structure that can store values of [`T`].
    /// # Safety
    ///
    /// The caller must ensure that the [`DataInfo`] that is stored for this `T` matces the actual
    /// memory layout of `T`, and that `DataInfo::drop_fn()` is safe to call with an [`OwningPtr`]  to `T`
    pub unsafe fn new_data_storage<T: Data>(&self) -> Option<BlobVec> {
        Some(BlobVec::new_for_data(
            self.id_map.get(&TypeId::of::<T>())?,
            1,
        ))
    }
}
