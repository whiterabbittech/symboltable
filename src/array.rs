use std::any::TypeId;
use std::collections::HashSet;

use typed_ids::SerialU64;

use super::Interner;

/// An [ArrayInterner] uses a [Vec] to intern [Symbol]s.
/// It performs `intern` in O(n), and `resolve` in O(1).
/// It has no memory optimizations, so every unique [String]
/// is stored exactly once in the table is stored without compression.
#[derive(Default, Clone, Debug)]
pub struct ArrayInterner {
    store: Vec<SymbolCell>,
}

impl ArrayInterner {
    pub fn new() -> Self {
        let store = vec![SymbolCell::new(String::from(""))];
        Self { store }
    }

    // returns the position of this string in the table,
    // offset by the empty block at position 0.
    fn position(&self, val: &String) -> Option<usize> {
        self.store
            .iter()
            .skip(1)
            .position(|cell| cell.value() == val)
            .map(|x| x + 1) // adjust position by one to account for the skip.
    }

    fn upsert_type(&mut self, position: usize, typ: TypeId) -> SerialU64<()> {
        let cell = self.store.get_mut(position).unwrap();
        if !cell.has_type(&typ) {
            cell.add_type(typ);
        }
        SerialU64::try_from(position as u64).unwrap()
    }

    fn get_type(&self, position: usize, typ: TypeId) -> Option<SerialU64<()>> {
        self.store
            .get(position)
            .map(|cell| cell.has_type(&typ))
            .and_then(|_| SerialU64::try_from(position as u64).ok())
    }

    fn add_new(&mut self, val: String, typ: TypeId) -> SerialU64<()> {
        let end = self.store.len();
        let mut cell = SymbolCell::new(val);
        cell.add_type(typ);
        self.store.push(cell);
        SerialU64::try_from(end as u64).unwrap()
    }
}

#[derive(Clone, Debug)]
struct SymbolCell {
    value: String,
    typs:  HashSet<TypeId>,
}

impl SymbolCell {
    pub fn new(value: String) -> Self {
        // Fill Slot[0] with an empty cell.
        Self {
            value,
            typs: Default::default(),
        }
    }

    fn value(&self) -> &String {
        &self.value
    }

    fn add_type(&mut self, id: TypeId) {
        self.typs.insert(id);
    }

    fn has_type(&self, id: &TypeId) -> bool {
        self.typs.contains(id)
    }
}

impl Interner for ArrayInterner {
    fn intern(&mut self, val: String, typ: TypeId) -> SerialU64<()> {
        // • To intern a string, we scan the vec to see if something matches.
        let index = self.position(&val);
        match index {
            // • If we find a match, check if the TypeId is already
            //   contained within. Otherwise, add it.
            Some(position) => self.upsert_type(position, typ),
            // If not found, append a new element to the end of the array.
            None => self.add_new(val, typ),
        }
    }

    fn resolve(&self, id: SerialU64<()>) -> String {
        let index = id.get() as usize;
        self.store.get(index).unwrap().value().clone()
    }

    fn get_interned(&self, val: String, typ: TypeId) -> Option<SerialU64<()>> {
        // We perform the same steps as intern, except we don't add the
        // string to the store, instead we check if the TypeId is already
        // contained within.
        self.position(&val)
            .and_then(|position| self.get_type(position, typ))
    }
}
