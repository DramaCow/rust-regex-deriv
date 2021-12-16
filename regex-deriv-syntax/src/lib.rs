#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
// #![warn(missing_docs)]

#[macro_use] extern crate lalrpop_util;

lalrpop_mod!(pub parser);

mod utils;

#[cfg(test)]
mod tests {
    use crate::parser;

    fn f(s: &'static str) -> Vec<bool> {
        s.chars().map(|c| {
            match c {
                '0' => false,
                '1' => true,
                _ => panic!(),
            }
        }).collect()
    }

    #[test]
    fn it_works() {
        let alphabet = vec!["a", "i", "u", "e", "o", "あ", "い", "う", "え", "お"];

        let regex = parser::CharClassParser::new().parse("[aiueo]").unwrap();
        for (c, is_in) in alphabet.iter().zip(f("1111100000")) {
            assert_eq!(is_in, regex.is_fullmatch(c));
        }
    
        let regex = parser::CharClassParser::new().parse("[あいうえお]").unwrap();
        for (c, is_in) in alphabet.iter().zip(f("0000011111")) {
            assert_eq!(is_in, regex.is_fullmatch(c));
        }

        let regex = parser::CharClassParser::new().parse("[aあ]").unwrap();
        for (c, is_in) in alphabet.iter().zip(f("1000010000")) {
            assert_eq!(is_in, regex.is_fullmatch(c));
        }
    }
}
