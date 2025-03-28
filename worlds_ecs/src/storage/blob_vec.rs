//! Most of this file is copied directly from <https://github.com/bevyengine/bevy/blob/main/crates/bevy_utils/src/lib.rs>
//! or <https://github.com/bevyengine/bevy/blob/main/crates/bevy_ecs/src/storage/blob_vec.rs>
#![allow(dead_code)]

use std::mem::ManuallyDrop;

use std::{
    alloc::{Layout, handle_alloc_error},
    cell::UnsafeCell,
    num::NonZeroUsize,
    ptr::NonNull,
};

use bevy_ptr::{OwningPtr, Ptr, PtrMut};

use crate::world::data::DataInfo;

/// Item that's generic over some function. That function will be called when the item is dropped.
pub struct OnDrop<F: FnOnce()> {
    callback: ManuallyDrop<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    /// Returns an object that will invoke the specified callback when dropped.
    pub fn new(callback: F) -> Self {
        Self {
            callback: ManuallyDrop::new(callback),
        }
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        // SAFETY: We may move out of `self`, since this instance can never be observed after it's dropped.
        let callback = unsafe { ManuallyDrop::take(&mut self.callback) };
        callback();
    }
}

/// A flat, type-erased data storage type
///
/// Used to densely store homogeneous ECS data. A blob is usually just an arbitrary block of contiguous memory without any identity, and
/// could be used to represent any arbitrary data (i.e. string, arrays, etc). This type is an extendable and re-allocatable blob, which makes
/// it a blobby Vec, a `BlobVec`.
#[derive(Clone)]
pub struct BlobVec {
    item_layout: Layout,
    capacity: usize,
    /// Number of elements, not bytes
    len: usize,
    // the `data` ptr's layout is always `array_layout(item_layout, capacity)`
    data: NonNull<u8>,
    // None if the underlying type doesn't need to be dropped
    drop: Option<unsafe fn(OwningPtr<'_>)>,
}

// We want to ignore the `drop` field in our `Debug` impl
impl std::fmt::Debug for BlobVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobVec")
            .field("item_layout", &self.item_layout)
            .field("capacity", &self.capacity)
            .field("len", &self.len)
            .field("data", &self.data)
            .finish()
    }
}

impl BlobVec {
    /// Creates a new [`BlobVec`] with the specified `capacity`.
    ///
    /// `drop` is an optional function pointer that is meant to be invoked when any element in the [`BlobVec`]
    /// should be dropped. For all Rust-based types, this should match 1:1 with the implementation of [`Drop`]
    /// if present, and should be `None` if `T: !Drop`. For non-Rust based types, this should match any cleanup
    /// processes typically associated with the stored element.
    ///
    /// # Safety
    ///
    /// `drop` should be safe to call with an [`OwningPtr`] pointing to any item that's been pushed into this [`BlobVec`].
    ///
    /// If `drop` is `None`, the items will be leaked. This should generally be set as None based on [`needs_drop`].
    ///
    /// [`needs_drop`]: core::mem::needs_drop
    pub unsafe fn new(
        item_layout: Layout,
        drop: Option<unsafe fn(OwningPtr<'_>)>,
        capacity: usize,
    ) -> BlobVec {
        let align = NonZeroUsize::new(item_layout.align()).expect("alignment must be > 0");
        let data = bevy_ptr::dangling_with_align(align);
        if item_layout.size() == 0 {
            BlobVec {
                data,
                // ZST `BlobVec` max size is `usize::MAX`, and `reserve_exact` for ZST assumes
                // the capacity is always `usize::MAX` and panics if it overflows.
                capacity: usize::MAX,
                len: 0,
                item_layout,
                drop,
            }
        } else {
            let mut blob_vec = BlobVec {
                data,
                capacity: 0,
                len: 0,
                item_layout,
                drop,
            };
            blob_vec.reserve_exact(capacity);
            blob_vec
        }
    }

    /// Creates a new [`BlobVec`] that stores a specific [`Data`] with the specified `capacity`.
    ///
    /// # Safety
    ///
    /// `data_info.drop_fn()` should be safe to call with an [`OwningPtr`] pointing to any item that's been pushed into this [`BlobVec`].
    ///
    /// If `data_info.drop_fn()` is `None`, the items will be leaked. This should generally be set as None based on [`needs_drop`].
    ///
    /// [`needs_drop`]: core::mem::needs_drop
    pub unsafe fn new_for_data(data_info: &DataInfo, capacity: usize) -> BlobVec {
        unsafe { BlobVec::new(data_info.layout(), data_info.drop_fn(), capacity) }
    }

    /// Returns the number of elements in the vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total number of elements the vector can hold without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the [`Layout`] of the element type stored in the vector.
    #[inline]
    pub fn layout(&self) -> Layout {
        self.item_layout
    }

    /// Reserves the minimum capacity for at least `additional` more elements to be inserted in the given `BlobVec`.
    /// After calling `reserve_exact`, capacity will be greater than or equal to `self.len() + additional`. Does nothing if
    /// the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it requests. Therefore, capacity can not be relied upon
    /// to be precisely minimal.
    ///
    /// # Panics
    ///
    /// Panics if new capacity overflows `usize`.
    pub fn reserve_exact(&mut self, additional: usize) {
        let available_space = self.capacity - self.len;
        if available_space < additional {
            // SAFETY: `available_space < additional`, so `additional - available_space > 0`
            let increment = unsafe { NonZeroUsize::new_unchecked(additional - available_space) };
            self.grow_exact(increment);
        }
    }

    /// Reserves the minimum capacity for at least `additional` more elements to be inserted in the given `BlobVec`.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        /// Similar to `reserve_exact`. This method ensures that the capacity will grow at least `self.capacity()` if there is no
        /// enough space to hold `additional` more elements.
        #[cold]
        fn do_reserve(slf: &mut BlobVec, additional: usize) {
            let increment = slf.capacity.max(additional - (slf.capacity - slf.len));
            let increment = NonZeroUsize::new(increment).unwrap();
            slf.grow_exact(increment);
        }

        if self.capacity - self.len < additional {
            do_reserve(self, additional);
        }
    }

    /// Grows the capacity by `increment` elements.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    /// For ZST it panics unconditionally because ZST `BlobVec` capacity
    /// is initialized to `usize::MAX` and always stays that way.
    fn grow_exact(&mut self, increment: NonZeroUsize) {
        let new_capacity = self
            .capacity
            .checked_add(increment.get())
            .expect("capacity overflow");
        let new_layout =
            array_layout(&self.item_layout, new_capacity).expect("array layout should be valid");
        let new_data = if self.capacity == 0 {
            // SAFETY:
            // - layout has non-zero size as per safety requirement
            unsafe { std::alloc::alloc(new_layout) }
        } else {
            // SAFETY:
            // - ptr was be allocated via this allocator
            // - the layout of the ptr was `array_layout(self.item_layout, self.capacity)`
            // - `item_layout.size() > 0` and `new_capacity > 0`, so the layout size is non-zero
            // - "new_size, when rounded up to the nearest multiple of layout.align(), must not overflow (i.e., the rounded value must be less than usize::MAX)",
            // since the item size is always a multiple of its align, the rounding cannot happen
            // here and the overflow is handled in `array_layout`
            unsafe {
                std::alloc::realloc(
                    self.get_ptr_mut().as_ptr(),
                    array_layout(&self.item_layout, self.capacity)
                        .expect("array layout should be valid"),
                    new_layout.size(),
                )
            }
        };

        self.data = NonNull::new(new_data).unwrap_or_else(|| handle_alloc_error(new_layout));
        self.capacity = new_capacity;
    }

    /// Initializes the value at `index` to `value`. This function does not do any bounds checking.
    ///
    /// # Safety
    /// - index must be in bounds
    /// - the memory in the [`BlobVec`] starting at index `index`, of a size matching this [`BlobVec`]'s
    /// `item_layout`, must have been previously allocated.
    #[inline]
    pub unsafe fn initialize_unchecked(&mut self, index: usize, value: OwningPtr<'_>) {
        debug_assert!(index < self.len());
        unsafe {
            let ptr = self.get_mut_unchecked(index);
            std::ptr::copy_nonoverlapping::<u8>(
                value.as_ptr(),
                ptr.as_ptr(),
                self.item_layout.size(),
            );
        }
    }

    /// Replaces the value at `index` with `value`. This function does not do any bounds checking.
    ///
    /// # Safety
    /// - index must be in-bounds
    /// - the memory in the [`BlobVec`] starting at index `index`, of a size matching this
    /// [`BlobVec`]'s `item_layout`, must have been previously initialized with an item matching
    /// this [`BlobVec`]'s `item_layout`
    /// - the memory at `*value` must also be previously initialized with an item matching this
    /// [`BlobVec`]'s `item_layout`
    pub unsafe fn replace_unchecked(&mut self, index: usize, value: OwningPtr<'_>) {
        debug_assert!(index < self.len());

        // Pointer to the value in the vector that will get replaced.
        // SAFETY: The caller ensures that `index` fits in this vector.
        let destination = NonNull::from(unsafe { self.get_mut_unchecked(index) });
        let source = value.as_ptr();

        if let Some(drop) = self.drop {
            // Temporarily set the length to zero, so that if `drop` panics the caller
            // will not be left with a `BlobVec` containing a dropped element within
            // its initialized range.
            let old_len = self.len;
            self.len = 0;

            // Transfer ownership of the old value out of the vector, so it can be dropped.
            // SAFETY:
            // - `destination` was obtained from a `PtrMut` in this vector, which ensures it is non-null,
            //   well-aligned for the underlying type, and has proper provenance.
            // - The storage location will get overwritten with `value` later, which ensures
            //   that the element will not get observed or double dropped later.
            // - If a panic occurs, `self.len` will remain `0`, which ensures a double-drop
            //   does not occur. Instead, all elements will be forgotten.
            let old_value = unsafe { OwningPtr::new(destination) };

            // This closure will run in case `drop()` panics,
            // which ensures that `value` does not get forgotten.
            let on_unwind = OnDrop::new(|| unsafe { drop(value) });

            unsafe {
                drop(old_value);
            }

            // If the above code does not panic, make sure that `value` doesn't get dropped.
            core::mem::forget(on_unwind);

            // Make the vector's contents observable again, since panics are no longer possible.
            self.len = old_len;
        }

        // Copy the new value into the vector, overwriting the previous value.
        // SAFETY:
        // - `source` and `destination` were obtained from `OwningPtr`s, which ensures they are
        //   valid for both reads and writes.
        // - The value behind `source` will only be dropped if the above branch panics,
        //   so it must still be initialized and it is safe to transfer ownership into the vector.
        // - `source` and `destination` were obtained from different memory locations,
        //   both of which we have exclusive access to, so they are guaranteed not to overlap.
        unsafe {
            std::ptr::copy_nonoverlapping::<u8>(
                source,
                destination.as_ptr(),
                self.item_layout.size(),
            );
        }
    }

    /// Appends an element to the back of the vector.
    ///
    /// # Safety
    /// The `value` must match the [`layout`](`BlobVec::layout`) of the elements in the [`BlobVec`].
    #[inline]
    pub unsafe fn push(&mut self, value: OwningPtr<'_>) {
        self.reserve(1);
        let index = self.len;
        self.len += 1;
        unsafe {
            self.initialize_unchecked(index, value);
        }
    }

    /// Forces the length of the vector to `len`.
    ///
    /// # Safety
    /// `len` must be <= `capacity`. if length is decreased, "out of bounds" items must be dropped.
    /// Newly added items must be immediately populated with valid values and length must be
    /// increased. For better unwind safety, call [`BlobVec::set_len`] _after_ populating a new
    /// value.
    #[inline]
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.capacity());
        self.len = len;
    }

    /// Performs a "swap remove" at the given `index`, which removes the item at `index` and moves
    /// the last item in the [`BlobVec`] to `index` (if `index` is not the last item). It is the
    /// caller's responsibility to drop the returned pointer, if that is desirable.
    ///
    /// # Safety
    /// It is the caller's responsibility to ensure that `index` is less than `self.len()`.
    #[inline]
    #[must_use = "The returned pointer should be used to dropped the removed element"]
    pub unsafe fn swap_remove_and_forget_unchecked(&mut self, index: usize) -> OwningPtr<'_> {
        debug_assert!(index < self.len());
        // Since `index` must be strictly less than `self.len` and `index` is at least zero,
        // `self.len` must be at least one. Thus, this cannot underflow.
        let new_len = self.len - 1;
        let size = self.item_layout.size();
        if index != new_len {
            unsafe {
                std::ptr::swap_nonoverlapping::<u8>(
                    self.get_mut_unchecked(index).as_ptr(),
                    self.get_mut_unchecked(new_len).as_ptr(),
                    size,
                );
            }
        }
        self.len = new_len;
        // Cannot use get_unchecked here as this is technically out of bounds after changing len.
        // SAFETY:
        // - `new_len` is less than the old len, so it must fit in this vector's allocation.
        // - `size` is a multiple of the erased type's alignment,
        //   so adding a multiple of `size` will preserve alignment.
        unsafe { self.get_ptr_mut().byte_add(new_len * size).promote() }
    }

    /// Removes the value at `index` and copies the value stored into `ptr`.
    /// Does not do any bounds checking on `index`.
    /// The removed element is replaced by the last element of the `BlobVec`.
    ///
    /// # Safety
    /// It is the caller's responsibility to ensure that `index` is < `self.len()`
    /// and that `self[index]` has been properly initialized.
    #[inline]
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize, ptr: PtrMut<'_>) {
        debug_assert!(index < self.len());
        unsafe {
            let last = self.get_mut_unchecked(self.len - 1).as_ptr();
            let target = self.get_mut_unchecked(index).as_ptr();
            // Copy the item at the index into the provided ptr
            std::ptr::copy_nonoverlapping::<u8>(target, ptr.as_ptr(), self.item_layout.size());
            // Recompress the storage by moving the previous last element into the
            // now-free row overwriting the previous data. The removed row may be the last
            // one so a non-overlapping copy must not be used here.
            std::ptr::copy::<u8>(last, target, self.item_layout.size());
        }
        // Invalidate the data stored in the last row, as it has been moved
        self.len -= 1;
    }

    /// Removes the value at `index` and drops it.
    /// Does not do any bounds checking on `index`.
    /// The removed element is replaced by the last element of the `BlobVec`.
    ///
    /// # Safety
    /// It is the caller's responsibility to ensure that `index` is `< self.len()`.
    #[inline]
    pub unsafe fn swap_remove_and_drop_unchecked(&mut self, index: usize) {
        debug_assert!(index < self.len());
        let drop = self.drop;
        let value = unsafe { self.swap_remove_and_forget_unchecked(index) };
        if let Some(drop) = drop {
            unsafe {
                drop(value);
            }
        }
    }

    /// Returns a reference to the element at `index`, without doing bounds checking.
    ///
    /// # Safety
    /// It is the caller's responsibility to ensure that `index < self.len()`.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> Ptr<'_> {
        debug_assert!(index < self.len());
        let size = self.item_layout.size();
        // SAFETY:
        // - The caller ensures that `index` fits in this vector,
        //   so this operation will not overflow the original allocation.
        // - `size` is a multiple of the erased type's alignment,
        //  so adding a multiple of `size` will preserve alignment.
        unsafe { self.get_ptr().byte_add(index * size) }
    }

    /// Returns a mutable reference to the element at `index`, without doing bounds checking.
    ///
    /// # Safety
    /// It is the caller's responsibility to ensure that `index < self.len()`.
    #[inline]
    pub unsafe fn get_mut_unchecked(&mut self, index: usize) -> PtrMut<'_> {
        debug_assert!(index < self.len());
        let size = self.item_layout.size();
        // SAFETY:
        // - The caller ensures that `index` fits in this vector,
        //   so this operation will not overflow the original allocation.
        // - `size` is a multiple of the erased type's alignment,
        //  so adding a multiple of `size` will preserve alignment.
        unsafe { self.get_ptr_mut().byte_add(index * size) }
    }

    /// Gets a [`Ptr`] to the start of the vec
    #[inline]
    pub fn get_ptr(&self) -> Ptr<'_> {
        // SAFETY: the inner data will remain valid for as long as 'self.
        unsafe { Ptr::new(self.data) }
    }

    /// Gets a [`PtrMut`] to the start of the vec
    #[inline]
    pub fn get_ptr_mut(&mut self) -> PtrMut<'_> {
        // SAFETY: the inner data will remain valid for as long as 'self.
        unsafe { PtrMut::new(self.data) }
    }

    /// Get a reference to the entire [`BlobVec`] as if it were an array with elements of type `T`
    ///
    /// # Safety
    /// The type `T` must be the type of the items in this [`BlobVec`].
    pub unsafe fn get_slice<T>(&self) -> &[UnsafeCell<T>] {
        // SAFETY: the inner data will remain valid for as long as 'self.
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const UnsafeCell<T>, self.len) }
    }

    /// Clears the vector, removing (and dropping) all values.
    ///
    /// Note that this method has no effect on the allocated capacity of the vector.
    pub fn clear(&mut self) {
        let len = self.len;
        // We set len to 0 _before_ dropping elements for unwind safety. This ensures we don't
        // accidentally drop elements twice in the event of a drop impl panicking.
        self.len = 0;
        if let Some(drop) = self.drop {
            let size = self.item_layout.size();
            for i in 0..len {
                // SAFETY:
                // * 0 <= `i` < `len`, so `i * size` must be in bounds for the allocation.
                // * `size` is a multiple of the erased type's alignment,
                //   so adding a multiple of `size` will preserve alignment.
                // * The item is left unreachable so it can be safely promoted to an `OwningPtr`.
                // NOTE: `self.get_unchecked_mut(i)` cannot be used here, since the `debug_assert`
                // would panic due to `self.len` being set to 0.
                let item = unsafe { self.get_ptr_mut().byte_add(i * size).promote() };
                // SAFETY: `item` was obtained from this `BlobVec`, so its underlying type must match `drop`.
                unsafe { drop(item) };
            }
        }
    }
}

impl Drop for BlobVec {
    fn drop(&mut self) {
        self.clear();
        let array_layout =
            array_layout(&self.item_layout, self.capacity).expect("array layout should be valid");
        if array_layout.size() > 0 {
            // SAFETY: data ptr layout is correct, swap_scratch ptr layout is correct
            unsafe {
                std::alloc::dealloc(self.get_ptr_mut().as_ptr(), array_layout);
            }
        }
    }
}

/// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
fn array_layout(layout: &Layout, n: usize) -> Option<Layout> {
    let (array_layout, offset) = repeat_layout(layout, n)?;
    debug_assert_eq!(layout.size(), offset);
    Some(array_layout)
}

// TODO: replace with `Layout::repeat` if/when it stabilizes
/// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
fn repeat_layout(layout: &Layout, n: usize) -> Option<(Layout, usize)> {
    // This cannot overflow. Quoting from the invariant of Layout:
    // > `size`, when rounded up to the nearest multiple of `align`,
    // > must not overflow (i.e., the rounded value must be less than
    // > `usize::MAX`)
    let padded_size = layout.size() + padding_needed_for(layout, layout.align());
    let alloc_size = padded_size.checked_mul(n)?;

    // SAFETY: self.align is already known to be valid and alloc_size has been
    // padded already.
    unsafe {
        Some((
            Layout::from_size_align_unchecked(alloc_size, layout.align()),
            padded_size,
        ))
    }
}

/// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
const fn padding_needed_for(layout: &Layout, align: usize) -> usize {
    let len = layout.size();

    // Rounded up value is:
    //   len_rounded_up = (len + align - 1) & !(align - 1);
    // and then we return the padding difference: `len_rounded_up - len`.
    //
    // We use modular arithmetic throughout:
    //
    // 1. align is guaranteed to be > 0, so align - 1 is always
    //    valid.
    //
    // 2. `len + align - 1` can overflow by at most `align - 1`,
    //    so the &-mask with `!(align - 1)` will ensure that in the
    //    case of overflow, `len_rounded_up` will itself be 0.
    //    Thus the returned padding, when added to `len`, yields 0,
    //    which trivially satisfies the alignment `align`.
    //
    // (Of course, attempts to allocate blocks of memory whose
    // size and padding overflow in the above manner should cause
    // the allocator to yield an error anyway.)

    let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
    len_rounded_up.wrapping_sub(len)
}
