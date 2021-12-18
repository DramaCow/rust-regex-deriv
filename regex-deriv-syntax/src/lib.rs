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

    fn test_cases(inputs: &[&str], cases: &[(&str, &str)]) {
        for (pattern, bitmap) in cases {
            let regex = parser::ExprParser::new().parse(pattern).unwrap();
            let answers = bitmap.chars().map(|c| {
                match c {
                    '0' => false,
                    '1' => true,
                    _ => panic!(),
                }
            });
            for (input, is_in) in inputs.iter().cloned().zip(answers) {
                if is_in {
                    assert!(regex.is_fullmatch(input), "Pattern r\"{}\" failed to recognize \"{}\".", pattern, input);
                } else {
                    assert!(!regex.is_fullmatch(input), "Pattern r\"{}\" should not recognize \"{}\".", pattern, input);
                }
            }
        }
    }

    #[test]
    fn it_works() {
        let inputs = vec!["a", "i", "u", "e", "o", "あ", "い", "う", "え", "お"];

        let cases = [
            ("[aiueo]", "1111100000"),
            ("[あいうえお]", "0000011111"),
            ("[aあ]", "1000010000"),
            ("a | あ", "1000010000"),
            ("a | i | [あ い]", "1100011000"),
            ("[aあ iい uう eえ oお] & [あいうえお]", "0000011111"),
            ("~a", "0111111111"),
        ];

        test_cases(&inputs, &cases);
    }
}
