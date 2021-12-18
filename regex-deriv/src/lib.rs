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

// =================
// === INTERNALS ===
// =================

#[cfg(test)]
mod tests;