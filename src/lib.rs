//! This crate provides generic escaping of characters without requiring allocations. It leverages
//! `fast_fmt` crate to do this.
//!
//! #Examples
//!
//! Escaping whole writer
//!
//! ```
//! #[macro_use]
//! extern crate fast_fmt;
//! extern crate fast_escape;
//! extern crate void;
//!
//! use fast_escape::Escaper;
//! use fast_fmt::Write;
//! use void::ResultVoidExt;
//!
//! fn main() {
//!     let mut s = String::new();
//!     {
//!         let s = &mut s;
//!         let mut tr = s.transform(Escaper::new('\\', '$'));
//!     
//!         fwrite!(&mut tr, "abcd$efgh").void_unwrap();
//!     }
//!     
//!     assert_eq!(s, "abcd\\$efgh");
//! }
//! ```
//!
//! Escaping part of formatted text
//!
//! ```
//! #[macro_use]
//! extern crate fast_fmt;
//! extern crate fast_escape;
//! extern crate void;
//!
//! use fast_escape::Escaper;
//! use void::ResultVoidExt;
//!
//! fn main() {
//!     let mut s = String::new();
//!     let special_chars = ['$', '"'];
//!     let escaper: Escaper<&[char]> = Escaper::new('\\', &special_chars);
//!     let value = "$Hello \"world\"!";
//!     fwrite!(&mut s, "$foo=\"", value.transformed(escaper), "\"").void_unwrap();
//!
//!     assert_eq!(s, "$foo=\"\\$Hello \\\"world\\\"!\"");
//! }
//! ```

#![no_std]

#[cfg_attr(test, macro_use)]
extern crate fast_fmt;

#[cfg(feature = "std")]
extern crate std;

/// Represents set of chars used for configuring `Escaper`.
pub trait ContainsChar {
    /// Returns true if the set represented by the type contains `c`.
    fn contains_char(&self, c: char) -> bool;

    /// Combinator for creating unions of the sets.
    fn union<T: ContainsChar>(self, other: T) -> Union<Self, T> where Self: Sized {
        Union::new(self, other)
    }
}

impl<'a, T: ContainsChar + ?Sized> ContainsChar for &'a T {
    fn contains_char(&self, c: char) -> bool {
        (*self).contains_char(c)
    }
}

impl ContainsChar for char {
    fn contains_char(&self, c: char) -> bool {
        c == *self
    }
}

impl ContainsChar for [char] {
    fn contains_char(&self, c: char) -> bool {
        self.contains(&c)
    }
}

impl ContainsChar for core::ops::Range<char> {
    fn contains_char(&self, c: char) -> bool {
        c >= self.start && c < self.end
    }
}

impl ContainsChar for core::ops::RangeFrom<char> {
    fn contains_char(&self, c: char) -> bool {
        c >= self.start
    }
}

impl ContainsChar for core::ops::RangeTo<char> {
    fn contains_char(&self, c: char) -> bool {
        c < self.end
    }
}

impl ContainsChar for core::ops::RangeFull {
    fn contains_char(&self, _: char) -> bool {
        true
    }
}

#[cfg(feature = "std")]
impl<S: std::hash::BuildHasher> ContainsChar for std::collections::HashSet<char, S> {
    fn contains_char(&self, c: char) -> bool {
        self.contains(&c)
    }
}

#[cfg(feature = "std")]
impl ContainsChar for std::collections::BTreeSet<char> {
    fn contains_char(&self, c: char) -> bool {
        self.contains(&c)
    }
}

/// Union of two sets of chars.
pub struct Union<A: ContainsChar, B: ContainsChar> {
    a: A,
    b: B,
}

impl<A: ContainsChar, B: ContainsChar> Union<A, B> {
    fn new(a: A, b: B) -> Self {
        Union {
            a,
            b
        }
    }
}

impl<A: ContainsChar, B: ContainsChar> ContainsChar for Union<A, B> {
    fn contains_char(&self, c: char) -> bool {
        self.a.contains_char(c) || self.b.contains_char(c)
    }
}

/// Set defined by given predicate (function).
pub struct Predicate<F: Fn(char) -> bool>(pub F);

impl<F: Fn(char) -> bool> ContainsChar for Predicate<F> {
    fn contains_char(&self, c: char) -> bool {
        self.0(c)
    }
}

/// This struct provides escaping of characters.
pub struct Escaper<C: ContainsChar> {
    chars: C,
    escape: char,
}

impl <C: ContainsChar> Escaper<C> {
    /// Creates the escaper.
    /// `escape_char` is the char which is used for escaping (e.g. '\\')
    /// `special_chars` is set of chars that should be escaped.
    pub fn new(escape_char: char, special_chars: C) -> Self {
        Escaper {
            chars: special_chars,
            escape: escape_char,
        }
    }
}

impl<C: ContainsChar> fast_fmt::transform::Transform for Escaper<C> {
    fn transform_char<W: fast_fmt::Write>(&self, writer: &mut W, c: char) -> Result<(), W::Error> {
        if self.chars.contains_char(c) {
            writer.write_char(self.escape)?;
        }
        writer.write_char(c)
    }

    fn transform_size_hint(&self, bytes: usize) -> usize {
        bytes * self.escape.len_utf8()
    }
}

#[cfg(test)]
mod tests {
    use ::Escaper;
    use fast_fmt::Write;
    use ::std::string::String;

    #[test]
    fn single_char() {
        let mut s = String::new();
        {
            let s = &mut s;
            let mut tr = s.transform(Escaper::new('\\', '$'));

            fwrite!(&mut tr, "abcd$efgh").unwrap();
        }

        assert_eq!(s, "abcd\\$efgh");
    }

    #[test]
    fn range() {
        let mut s = String::new();
        {
            let s = &mut s;
            let mut tr = s.transform(Escaper::new('\\', 'a'..'c'));

            fwrite!(&mut tr, "abcd$efgh").unwrap();
        }

        assert_eq!(s, "\\a\\bcd$efgh");
    }

    #[test]
    fn union() {
        use ::ContainsChar;

        let mut s = String::new();
        {
            let s = &mut s;
            let mut tr = s.transform(Escaper::new('\\', ('a'..'c').union('e'..'g')));

            fwrite!(&mut tr, "abcd$efgh").unwrap();
        }

        assert_eq!(s, "\\a\\bcd$\\e\\fgh");
    }
}
