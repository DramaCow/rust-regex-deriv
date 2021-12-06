//! This module is currently in early development and is highly unstable.
//! Use is not yet recommended.

#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
#![warn(missing_docs)]

mod byte_set;
pub use self::byte_set::ByteSet;

mod regex;
pub use self::regex::{
    RegEx,
    Operator,
};

mod dfa;
pub use self::dfa::DFA;

mod table;
pub use self::table::{
    LexTable,
    NaiveLexTable,
};

mod scan;
pub use self::scan::{
    Token,
    Scan,
    ScanError
};

/// Constructs a `RegEx` that recognizes some input string only.
#[must_use]
pub fn literal(s: &str) -> RegEx {
    s.bytes().fold(RegEx::empty(), |r, byte| {
        r.then(&RegEx::set(ByteSet::point(byte)))
    })
}

/// Constructs a `RegEx` that recognizes any char in a string.
#[must_use]
pub fn any(s: &str) -> RegEx {
    s.chars().fold(RegEx::empty(), |r, c| {
        let mut buffer: [u8; 4] = [0; 4];
        r.or(&literal(c.encode_utf8(&mut buffer)))
    })
}

// Constructs a `RegEx` that recognizes all chars within a provided range (inclusive).
// Also accounts for char ranges that span different number of bytes.

// =================
// === INTERNALS ===
// =================

#[cfg(test)]
mod tests;