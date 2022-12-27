use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use typed_ids::SerialU64;

use crate::internable::Internable;
use crate::Interner;

/// A Symbol uniquely represents each String contained in
/// the [SymbolTable]. It serves as a lookup key into the table,
/// allowing anyone holding a [Symbol] to recover the interned
/// value, or to compare the interned value against other interned
/// values of the same type. These comparisons are O(1).  
#[derive(Clone)]
pub struct Symbol<T: Internable + 'static> {
    // This ID maps the Symbol to an entry in the table.
    id:     SerialU64<T>,
    // This is a reference to the table storing the Symbol.
    lookup: Rc<dyn Resolvable>,
}

impl<T: Internable + 'static> Symbol<T> {
    /// [new] will construct a new Symbol. This method is only
    /// intended for internal use.
    pub fn new<R: Resolvable + 'static>(id: SerialU64<T>, lookup: R) -> Self {
        let lookup = Rc::new(lookup);
        Self { id, lookup }
    }

    pub fn id(&self) -> SerialU64<T> {
        self.id
    }

    pub fn erase_type(&self) -> SerialU64<()> {
        let id_unwrapped = self.id.get();
        SerialU64::<()>::try_from(id_unwrapped).unwrap()
    }

    pub fn origin(&self) -> *const (dyn Interner + 'static) {
        self.lookup.addr()
    }

    /// # Panics
    /// This method panics if the recovered string cannot be
    /// parsed back into the type that generated it.
    fn into(&self) -> T {
        let erased = self.erase_type();
        let interned_string = self.lookup.resolve(erased);
        let value = T::try_from(interned_string);
        match value {
            Ok(item) => item,
            Err(_) => panic!("Interned value was not recoverable."),
        }
    }
}

impl<T: Internable + 'static> fmt::Display for Symbol<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let recovered_item: T = self.into();
        let as_string: String = recovered_item.as_ref().to_string();
        write!(f, "{}", as_string)
    }
}

impl<T: Internable + 'static> fmt::Debug for Symbol<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<T: Internable + 'static> PartialEq for Symbol<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.lookup.addr() == other.lookup.addr()
    }
}

impl<T: Internable + 'static> Eq for Symbol<T> {}

impl<T: Internable + 'static> Hash for Symbol<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: Internable + 'static> Ord for Symbol<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T: Internable + 'static> PartialOrd for Symbol<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// This attempt to practice the "Accept Interface, Return Structs"
// best-practice. Symbols need to hold reference to a [SymbolTable], but
// [Symbol]s might outlive their table, so they need to hold a reference-counted
// pointer to the table.
/// A type is resolvable if it implements the resolution API for
/// [Interner]s.
pub trait Resolvable {
    fn resolve(&self, id: SerialU64<()>) -> String;

    /// This function returns the address of the backing table.
    /// This allows [Symbol]s to ensure they are being compared
    /// against the table from which they originated.
    fn addr(&self) -> *const (dyn Interner + 'static);
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_obj_safe;

    use super::Resolvable;

    #[test]
    fn resolvable_is_obj_safe() {
        assert_obj_safe!(Resolvable);
    }
}
