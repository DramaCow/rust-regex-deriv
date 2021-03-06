#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
// #![warn(missing_docs)]

use regex_deriv::{RegEx, ByteSet};
#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(parser);
mod utils;

pub type ParseError<'a> =
    lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'a>, &'static str>;

pub fn parse(pattern: &str) -> Result<RegEx, ParseError> {
    parser::ExprParser::new().parse(pattern)
}

// Constructs a `RegEx` that recognizes some input string only.
pub fn literal(s: &str) -> RegEx {
    s.bytes().fold(RegEx::empty(), |r, byte| {
        r.then(&RegEx::set(ByteSet::point(byte)))
    })
}