use std::collections::VecDeque;
use std::fmt;

use super::internable::Internable;
use super::Symbol;

#[derive(Clone)]
pub struct SymbolIterator<T: Internable + 'static> {
    source:    Symbol<T>,
    remaining: VecDeque<char>,
}

impl<T: Internable + 'static> PartialEq for SymbolIterator<T> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.remaining == other.remaining
    }
}

impl<T: Internable + 'static> SymbolIterator<T> {
    pub fn new(source: Symbol<T>) -> Self {
        let remaining: VecDeque<char> = source.to_string().chars().collect();
        Self { source, remaining }
    }

    pub fn has_next(&self) -> bool {
        !self.remaining.is_empty()
    }

    pub fn peek(&self) -> Option<char> {
        self.remaining.front().cloned()
    }
}

impl<T: Internable + 'static> Iterator for SymbolIterator<T> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.remaining.pop_front()
    }
}

impl<T: Internable + 'static> DoubleEndedIterator for SymbolIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.remaining.pop_back()
    }
}

impl<T: Internable + 'static> fmt::Display for SymbolIterator<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T: Internable + 'static> fmt::Debug for SymbolIterator<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let original = self.source.to_string();
        let num_matched = original.len() - self.remaining.len();
        // Write the leading quote mark.
        write!(f, "\"")?;
        for (i, char) in original.chars().enumerate() {
            // Write the index pointer.
            if i == num_matched {
                write!(f, "•")?;
            }
            // Write the next character.
            write!(f, "{}", char)?;
        }
        // Special case: if the iterator is empty,
        // then we have to write the dot at the end.
        if !self.has_next() {
            write!(f, "•")?;
        }
        // Write the closing quote mark.
        write!(f, "\"")
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_str_eq};

    use super::super::{InternerFlavor, Symbol, SymbolTable};
    use super::SymbolIterator;

    fn toad_iter() -> SymbolIterator<String> {
        let mut table = SymbolTable::new(InternerFlavor::Array);
        let s = "toad".to_owned();
        let sym: Symbol<String> = table.intern(&s);
        SymbolIterator::new(sym)
    }

    #[test]
    fn neq() {
        let left = toad_iter();
        let right = toad_iter();
        assert_ne!(left, right);
    }

    #[test]
    fn eq() {
        let left = toad_iter();
        let mut right = left.clone();
        assert_eq!(left, right);
        right.next();
        assert_ne!(left, right);
    }

    #[test]
    fn display() {
        let mut characters = toad_iter();
        assert_str_eq!(characters.to_string(), "\"•toad\"");
        characters.next();
        assert_str_eq!(characters.to_string(), "\"t•oad\"");
        characters.next();
        assert_str_eq!(characters.to_string(), "\"to•ad\"");
        characters.next();
        assert_str_eq!(characters.to_string(), "\"toa•d\"");
        characters.next();
        assert_str_eq!(characters.to_string(), "\"toad•\"");
    }

    #[test]
    fn iterable() {
        let mut toad = toad_iter();
        assert!(toad.has_next());
        assert_eq!(toad.peek(), Some('t'));
        assert_eq!(toad.next(), Some('t'));
        assert!(toad.has_next());
        assert_eq!(toad.peek(), Some('o'));
        assert_eq!(toad.next(), Some('o'));
        assert!(toad.has_next());
        assert_eq!(toad.peek(), Some('a'));
        assert_eq!(toad.next(), Some('a'));
        assert!(toad.has_next());
        assert_eq!(toad.peek(), Some('d'));
        assert_eq!(toad.next(), Some('d'));
        assert!(!toad.has_next());
        assert_eq!(toad.peek(), None);
        assert_eq!(toad.next(), None);
    }
}
