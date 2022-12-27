//! This crate offers a [SymbolTable] type which can store strings as
//! lightweight [Symbols], which enable extremely fast comparison and total
//! order operations. Depending on the backing data structure, the [SymbolTable]
//! may also offer fast convertion from [Symbol] to [String]. Finally, [Symbol]
//! is parameterized by a type, allowing you to intern strings coming from
//! incomparable sources without the possibility of mixing them up.
//! For example, if you intern an Address: Into<String> and Username:
//! Into<String>, you can get back a Symbol<Address> and a Symbol<Username>.
//! These two [Symbol] types will share the same store and any benefits of
//! compression, while ensuring you don't mix up one Symbol for another, as is
//! easy with strings: ```text
//! fn foo(address: String, username: String);
//! foo(my_username, my_address); // This is well-typed, but is logically
//! erronious, because the parameters were mixed up.
//! fn foo2(address: Symbol<Address>, username: Symbol<Username>);
//! // This formulation would produce an type error when you accidently
//! // swap the argument positions.
//! ```
use array::ArrayInterner;
pub use errors::{ResolutionErr, TableMismatchErr};
pub use flavor::InternerFlavor;
pub use internable::Internable;
pub use interner::Interner;
use symbol::Resolvable;
pub use symbol::Symbol;
pub use symbol_iterator::SymbolIterator;
pub use table::SymbolTable;

mod array;
mod errors;
mod flavor;
mod internable;
mod interner;
mod symbol;
mod symbol_iterator;
mod table;
