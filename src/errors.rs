use std::fmt;
use std::marker::PhantomData;

use colored::Colorize;
use miette::Diagnostic;
use thiserror::Error;

use crate::{Internable, Interner};

#[derive(Debug, PartialEq, Eq)]
pub enum ResolutionErr<T: Internable + 'static> {
    MismatchErr(TableMismatchErr<T>),
    ParseErr,
}

impl<T: Internable + 'static> From<TableMismatchErr<T>> for ResolutionErr<T> {
    fn from(val: TableMismatchErr<T>) -> Self {
        ResolutionErr::MismatchErr(val)
    }
}

// TODO Fix error defintion.
/// [ResolutionErr] occurs when a [Symbol] is resolved on a [SymbolTable] from
/// which it did not originate. If a user creates two separate [SymbolTable]s,
/// a [Symbol] from one table is not available to be resolved by the other
/// table. If a user attempts this, a [ResolutionErr] is returned, indicating
/// that the identities of the two tables are different.
#[derive(PartialEq, Eq, Error, Diagnostic)]
#[error(
    "This Symbol did not originate from this table. The Symbol's originator has the address xxxx \
     but this table's address is xxxx"
)]
pub struct TableMismatchErr<T: Internable + 'static> {
    table_address:  *const (dyn Interner + 'static),
    symbol_address: *const (dyn Interner + 'static),
    data:           PhantomData<T>,
}

impl<T: Internable + 'static> TableMismatchErr<T> {
    pub fn new(
        table: *const (dyn Interner + 'static),
        sym: *const (dyn Interner + 'static),
    ) -> Self {
        Self {
            table_address:  table,
            symbol_address: sym,
            data:           PhantomData,
        }
    }
}

impl<T: Internable + 'static> fmt::Debug for TableMismatchErr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let table_msg = format!("{:?}", self.table_address).red().bold();
        let sym_msg = format!("{:?}", self.symbol_address).red().bold();
        write!(
            f,
            "This Symbol did not originate from this table. The Symbol's originator has the \
             address {} but this table's address is {}",
            sym_msg, table_msg
        )
    }
}
