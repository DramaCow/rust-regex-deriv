use regex_deriv_syntax as re;

mod common;

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

        common::test_cases(&inputs, &cases);
    }