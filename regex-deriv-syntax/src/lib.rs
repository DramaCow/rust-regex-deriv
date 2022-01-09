#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
// #![warn(missing_docs)]

use regex_deriv::RegEx;
#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(parser);
mod utils;

pub type ParseError<'a> =
    lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'a>, &'static str>;

pub fn parse(pattern: &str) -> Result<RegEx, ParseError> {
    parser::ExprParser::new().parse(pattern)
}