use std::any::TypeId;
use std::cell::RefCell;
use std::rc::Rc;

use typed_ids::SerialU64;

use super::{
    ArrayInterner, Internable, Interner, InternerFlavor, ResolutionErr, Resolvable, Symbol,
    TableMismatchErr,
};

/// A [SymbolTable] allows you to store items according to their
/// [String] representation in a lookup table. The lookup table
/// typically compresses the strings to save space on large tables.
/// The [SymbolTable] provides a handy, opaque ID for each entry in
/// the table, called a [Symbol]. This [Symbol] allows for O(1) comparison
/// of strings because the table is responsible for encoding string
/// uniqueness into each id.
#[derive(Clone)]
pub struct SymbolTable {
    // What if I pass the type ID into the interner?
    // Map the string to the typeID provided.
    interner: Rc<RefCell<dyn Interner>>,
}

impl SymbolTable {
    pub fn new(flavor: InternerFlavor) -> Self {
        match flavor {
            InternerFlavor::Array => Self::from(ArrayInterner::new()),
        }
    }

    /// The [intern] function takes any object which can be converted
    /// to and from a [String], and interns it into the table. The resulting
    /// [Symbol] is unique if and only if no other item with the same type
    /// has already been stored in the table.
    pub fn intern<T: Internable>(&mut self, item: &T) -> Symbol<T> {
        // • Take this item and convert it into a string.
        let str_repr: String = item.as_ref().to_string();
        // • Fetch the type id, which we'll need to differentiate
        //   stored items of type T from stored items of type T'.
        let typ_id = TypeId::of::<T>();
        // • Now that we have both the Type Id and the String representation,
        //   we can intern the item in the data structure.
        let erased_id = self.interner.borrow_mut().intern(str_repr, typ_id);
        // • Now that we have the id of the entry, we need to convert
        //   this into a Symbol and increase the strength of the typing.
        self.to_typed_symbol(erased_id)
    }

    /// Resolve returns the object that was originally stored in the table.
    /// If this [Symbol] was created by a [SymbolTable] other than `self`, then
    /// [resolve] returns a [ResolutionErr]. Otherwise, a valid value will be
    /// returned.
    pub fn resolve<T: Internable + 'static>(&self, sym: &Symbol<T>) -> Result<T, ResolutionErr<T>> {
        // • Before we do anything else, we need to confirm this Symbol
        //   originates from this table. Check the pointer of this table
        //   makes the memory location of the Symbol's table.
        let table_addr = self.addr();
        let sym_addr = sym.origin();
        if table_addr != sym_addr {
            let err = ResolutionErr::from(TableMismatchErr::new(table_addr, sym_addr));
            return Err(err);
        }
        // • Convert the Symbol back into an Id.
        let id = sym.erase_type();
        let resolution = self.interner.borrow().resolve(id);
        // TODO: Don't throw away the result. Capture it in the ParseErr.
        T::try_from(resolution).map_err(|_| ResolutionErr::ParseErr)
    }

    pub fn get_interned<T: Internable + 'static, S: AsRef<str>>(
        &self,
        val: S,
    ) -> Option<Symbol<T>> {
        // • Get the string representation of the passed value.
        let str_repr: String = val.as_ref().to_string();
        // • Fetch the type id, which we'll need to differentiate
        //   stored items of type T from stored items of type T'.
        let typ_id = TypeId::of::<T>();
        // • Query the store, asking about this string and type.
        let id = self.interner.borrow().get_interned(str_repr, typ_id)?;
        // Convert the id into a Symbol.
        Some(self.to_typed_symbol(id))
    }

    pub fn has_interned<T: Internable + 'static, S: AsRef<str>>(&self, val: S) -> bool {
        self.get_interned::<T, S>(val).is_some()
    }

    fn to_typed_symbol<T: Internable>(&self, id: SerialU64<()>) -> Symbol<T> {
        let upcast_id = self.upcast(id);
        self.id_as_symbol(upcast_id)
    }

    fn upcast<T: Internable>(&self, id: SerialU64<()>) -> SerialU64<T> {
        SerialU64::<T>::try_from(id.get()).unwrap()
    }

    fn id_as_symbol<T: Internable>(&self, id: SerialU64<T>) -> Symbol<T> {
        Symbol::new(id, self.clone())
    }
}

// Symbols need a way to recover their String representation without
// the user passing in a reference to the SymbolTable. They do this by
// holding a reference to the table themselves.
// Due to mutability constraints, the Symbol doesn't accept a Rc<SymbolTable>
// directly, but instead mediates its API needs through the Resolvable trait.
// This eliminates mutability limitations between ref-counted table instances
// and the Symbols that hold those references.
impl Resolvable for SymbolTable {
    // To implement resolve, we delegate the work to
    // the held interner.
    fn resolve(&self, id: SerialU64<()>) -> String {
        self.interner.borrow().resolve(id)
    }

    // Here, we return the address of the underlying interner,
    // which is the only truly stable memory address.
    fn addr(&self) -> *const (dyn Interner + 'static) {
        self.interner.as_ptr()
    }
}

impl<T: Interner + 'static> From<T> for SymbolTable {
    fn from(interner: T) -> Self {
        let cell = RefCell::new(interner);
        let ref_counter = Rc::new(cell);
        Self {
            interner: ref_counter,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InternerFlavor, Symbol, SymbolTable};

    #[test]
    fn symbols_mismatch() {
        let mut table = SymbolTable::new(InternerFlavor::Array);
        let s1 = "hello".to_owned();
        let s2 = "goodbye".to_owned();
        let id1: Symbol<String> = table.intern(&s1);
        let id2 = table.intern(&s2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn symbol_belongs_to() {
        let mut table1 = SymbolTable::new(InternerFlavor::Array);
        let table2 = SymbolTable::new(InternerFlavor::Array);
        let s1 = "hello".to_owned();
        let id: Symbol<String> = table1.intern(&s1);
        let expected_err = table2.resolve(&id);
        if expected_err.is_ok() {
            assert!(false, "Expected error.")
        }
    }

    #[test]
    fn can_recover_symbols() {
        let mut table = SymbolTable::new(InternerFlavor::Array);
        let s1 = "hello".to_owned();
        let s2 = "goodbye".to_owned();
        let id1: Symbol<String> = table.intern(&s1);
        let id2: Symbol<String> = table.intern(&s2);

        assert_eq!(table.clone().resolve(&id1), Ok(s1));
        assert_eq!(table.clone().resolve(&id2), Ok(s2));
    }

    #[test]
    fn has_string() {
        let mut table = SymbolTable::new(InternerFlavor::Array);
        let s1 = "frog".to_owned();
        let _: Symbol<String> = table.intern(&s1);
        assert_eq!(true, table.has_interned::<String, _>("frog"));
        assert_eq!(false, table.has_interned::<String, _>("toad"));
    }
}
