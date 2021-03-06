use super::RegEx;
use super::ByteSet;
use super::DFA;
use super::NaiveLexTable;
use super::Scan;

#[test]
fn approx_eq() {
    let re1 = RegEx::set(ByteSet::range(3, 17).complement());
    let re2 = RegEx::set(ByteSet::range(3, 17)).not();
    assert_ne!(re1, re2);
}

#[test]
fn derivative() {
    let set1 = RegEx::set(ByteSet::range(0, 16));
    let set2 = RegEx::set(ByteSet::range(8, 24));
    let re1 = set1.and(&set2).or(&set1).and(&set2);
    let regex = re1.then(&re1).then(&re1).then(&re1);
    assert_eq!(regex.deriv(8).deriv(8).deriv(8).deriv(8), RegEx::empty());
    assert_eq!(regex.deriv(0), RegEx::none());
}

#[test]
fn simple_lexer() {
    let table = NaiveLexTable::new(&DFA::from(&[
        RegEx::set(ByteSet::point(b' ').union(&ByteSet::point(b','))).plus(),
        // RegEx::set(ByteSet::range('A' as u8, 'Z' as u8)),
        RegEx::set(ByteSet::range(b'a', b'z')).plus(),
    ]).minimize());
    let text = "waltz, bad nymph, for quick jigs vex";

    let tokens: Vec<_> = Scan::new(&table, &text).take(14).collect::<Result<_, _>>().unwrap();
    let tokens: Vec<_> = tokens.into_iter().filter(|token| token.class != 0).collect();

    assert_eq!(&text[tokens[0].span.clone()], "waltz");
    assert_eq!(&text[tokens[1].span.clone()], "bad");
    assert_eq!(&text[tokens[2].span.clone()], "nymph");
    assert_eq!(&text[tokens[3].span.clone()], "for");
    assert_eq!(&text[tokens[4].span.clone()], "quick");
    assert_eq!(&text[tokens[5].span.clone()], "jigs");
    assert_eq!(&text[tokens[6].span.clone()], "vex");
    assert!(tokens.iter().all(|token| token.class == 1));
}

#[test]
fn fail_lexer() {
    let table = NaiveLexTable::new(&DFA::from(&[
        RegEx::set(ByteSet::range(b'A', b'Z')),
    ]).minimize());
    let text = "bad";
    let res = Scan::new(&table, &text).next().unwrap();
    assert!(res.is_err());
}