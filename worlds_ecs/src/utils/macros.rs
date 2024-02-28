#[macro_export]
/// Macro to define simple methods on structs that are used to ID.
/// The struct must be a single-field tuple-struct.
macro_rules! impl_id_struct {
    ($name:ident) => {
        impl_id_struct!($name, usize, pub);
    };
    ($name:ident, $id_type:ty) => {
        impl_id_struct!($name, $id_type, pub);
    };
    ($name:ident, $id_type:ty, $vis:vis) => {
        #[allow(unused)]
        impl $name {
            /// Create a new [`Self`] from a raw id.
            #[inline]
            $vis fn new(id: $id_type) -> $name {
                Self(id)
            }

            /// Get the underlying id.
            #[inline]
            $vis fn id(&self) -> $id_type {
                self.0
            }
        }
    };
}
