/// A type is [Internable] if it supports conversion to and from
/// [String], and it is static. It doesn't always need to be parsable
/// from a string, but the output of .toString() must be parsable by TryFrom()
pub trait Internable: TryFrom<String> + AsRef<str> {}

/// This Blanket implementation allows any time that implements the
/// the type bounds of [Internable] to implicitly be [Internable].
impl<T: TryFrom<String> + AsRef<str>> Internable for T {}
