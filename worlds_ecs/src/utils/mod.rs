pub(crate) mod macros;

/// A specialized hashmap type with Key of [`TypeId`]
pub type TypeIdMap<V> =
    std::collections::HashMap<std::any::TypeId, V, std::hash::BuildHasherDefault<NoOpTypeIdHasher>>;

#[doc(hidden)]
#[derive(Default)]
pub struct NoOpTypeIdHasher(u64);

// TypeId already contains a high-quality hash, so skip re-hashing that hash.
impl std::hash::Hasher for NoOpTypeIdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        // This will never be called: TypeId always just calls write_u64 once!
        // This is a known trick and unlikely to change, but isn't officially guaranteed.
        // Don't break applications (slower fallback, just check in test):
        self.0 = bytes.iter().fold(self.0, |hash, b| {
            hash.rotate_left(8).wrapping_add(*b as u64)
        });
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}
