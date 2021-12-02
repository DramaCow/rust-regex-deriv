use super::RegEx;
use super::ByteSet;

#[test]
fn approx_eq() {
    let re1 = RegEx::set(ByteSet::range(3, 17).complement());
    let re2 = RegEx::set(ByteSet::range(3, 17)).not();
    assert_eq!(re1, re2);
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