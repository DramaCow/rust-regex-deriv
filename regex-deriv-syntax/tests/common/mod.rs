use regex as oracle; // the official rust regex library
use regex_deriv_syntax::parse;

pub fn test_cases(inputs: &[&str], cases: &[(&str, &str)]) {
    for (pattern, bitmap) in cases {
        let regex = parse(pattern).unwrap();
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