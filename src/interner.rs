use std::any::TypeId;

use typed_ids::SerialU64;

// Maps (String, TypeID) => SerialU64 / UUID
/// [Interner] is a backing store for the [SymbolTable]. It is responsible for
/// implementing [Symbol] uniqueness and [String] compression. You can provide
/// your own interner, or use one of the provided implementations. Most users
/// should expect to use one of the implementations provided by this library.
/// You should only expect to implement [Interner] yourself if the compression
/// algorithms are not suitable for your needs.
pub trait Interner {
    /// The [intern] function maps a [String] and the [TypeId] of the
    /// type of the interned value to an an id. This id must be unique
    /// if the String key is unique. Two Symbols can share the same
    /// SerialU64 and have different types if and only if they have the same
    /// String key.
    /// By making the SymbolTable responsible for strengthening the typing
    /// guarantees, the Interner is able to compress `n` types with
    /// the same string represention using O(1) memory.
    fn intern(&mut self, val: String, typ: TypeId) -> SerialU64<()>;
    fn resolve(&self, id: SerialU64<()>) -> String;
    /// [get_interned] returns the untyped id of the Symbol corresponding
    /// to the String, if the string is contained within the store.
    fn get_interned(&self, val: String, typ: TypeId) -> Option<SerialU64<()>>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_obj_safe;

    use super::Interner;

    #[test]
    fn internable_is_obj_safe() {
        assert_obj_safe!(Interner);
    }
}
