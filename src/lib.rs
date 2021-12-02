//! This module is currently in early development and is highly unstable.
//! Use is not yet recommended.

#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
#![warn(missing_docs)]

mod char_set;
pub use self::char_set::CharSet;

mod regex;
pub use self::regex::{
    RegEx,
    Operator,
};

mod dfa;
pub use self::dfa::DFA;

mod table;
pub use self::table::{
    Command,
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
        r.then(&RegEx::set(CharSet::point(byte)))
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

/// Constructs a `RegEx` that recognizes all chars within a provided range (inclusive).
/// Also accounts for char ranges that span different number of bytes.
#[must_use]
pub fn range(from: char, to: char) -> RegEx {
    fn range8(from: u8, to: u8) -> RegEx {
        RegEx::set(CharSet::range(from, to))
    }

    #[allow(clippy::cast_possible_truncation)]
    fn range32(from: u32, to: u32, optional: bool) -> RegEx {
        let (a_low, a_high) = (from as u8 + optional as u8, from >> 8);
        let (b_low, b_high) = (to as u8, to >> 8);
    
        let regex = {
            if b_high == 0 {
                range8(a_low, b_low)
            } else if a_high == b_high {
                range8(a_low, b_low).then(&range32(a_high, b_high, false))
            } else if a_low == u8::MIN && b_low == u8::MAX {
                range8(u8::MIN, u8::MAX).then(&range32(a_high, b_high, a_high == 0))
            } else if a_low == u8::MIN {
                range8(u8::MIN, b_low).then(&range32(a_high, b_high, a_high == 0))
                .or(&range8(b_low + 1, u8::MAX).then(&range32(a_high, b_high - 1, a_high == 0)))
            } else if b_low == u8::MAX {
                range8(u8::MIN, a_low - 1).then(&range32(a_high + 1, b_high, false))
                .or(&range8(a_low, u8::MAX).then(&range32(a_high, b_high, a_high == 0)))
            } else if b_low >= a_low {
                range8(u8::MIN, a_low - 1).then(&range32(a_high + 1, b_high, false))
                .or(&range8(a_low, b_low).then(&range32(a_high, b_high, a_high == 0)))
                .or(&range8(b_low + 1, u8::MAX).then(&range32(a_high, b_high - 1, a_high == 0)))
            } else if b_high > a_high + 1 && b_low + 1 < a_low {
                range8(u8::MIN, b_low).then(&range32(a_high + 1, b_high, false))
                .or(&range8(b_low + 1, a_low - 1).then(&range32(a_high + 1, b_high - 1, false)))
                .or(&range8(a_low, u8::MAX).then(&range32(a_high, b_high - 1, a_high == 0)))
            } else {
                range8(u8::MIN, b_low).then(&range32(a_high + 1, b_high, false))
                .or(&range8(a_low, u8::MAX).then(&range32(a_high, b_high - 1, a_high == 0)))
            }
        };
    
        if optional { regex.opt() } else { regex }
    }
    
    range32(from as u32, to as u32, false)
}

// =================
// === INTERNALS ===
// =================

#[cfg(test)]
mod tests;