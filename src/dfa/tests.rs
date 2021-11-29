#![allow(non_snake_case)]

use super::{RegEx, DFA, CharSet};

// #[test]
// fn test() {
//     let a = RegEx::set(CharSet::point(1));
//     let b = RegEx::set(CharSet::point(2));
//     let c = RegEx::set(CharSet::point(3));
//     let regex = a.or(&b.then(&a)).or(&c);
//     let dfa = DFA::from(&regex);
//     println!("{}", dfa.dot().unwrap());
// }

#[test]
fn excluding() {
    let digit = RegEx::set(CharSet::range(0x30, 0x39));
    let zero = RegEx::set(CharSet::point(0x30));
    let nonzero_digit = digit.and(&zero.not());

    let A = DFA::from(&nonzero_digit);

    assert!(!A.matches("0"));
    for i in 1..=9 {
        assert!(A.matches(&i.to_string()));
    }
}

#[test]
fn indentifiers() {
    let uppercase  = RegEx::set(CharSet::range(0x41, 0x5a));
    let lowercase  = RegEx::set(CharSet::range(0x61, 0x7a));
    let digit      = RegEx::set(CharSet::range(0x30, 0x39));
    let underscore = RegEx::set(CharSet::point(0x5f));

    let character  = uppercase.or(&lowercase);
    let indentifier = character.or(&underscore).then(&character.or(&digit).or(&underscore).star());
    // println!("{:?}", &indentifier);

    let A = DFA::from(&indentifier);

    assert!( !A.matches("") );
    assert!( !A.matches("123notanidentifier") );
    assert!( A.matches("ThIsIsAlLoWeD") );
    assert!( A.matches("__allowed_123_") );
    assert!( !A.matches("not allowed") );
}