use regex_deriv::RegEx;

// Constructs a `RegEx` that recognizes all chars within a provided range (inclusive).
// Also accounts for char ranges that span different number of bytes. Inputs must be
// valid single unicode chars (as string slices).
pub fn range(a: &str, b: &str) -> RegEx {
    let mut a_chars = a.chars();
    let mut b_chars = b.chars();

    let from = a_chars.next().unwrap() as u32;
    let to = b_chars.next().unwrap() as u32;
    
    assert!(a_chars.next().is_none());
    assert!(b_chars.next().is_none());
    
    RegEx::range32(from, to)
}