/// Compactly represents a set of 8-bit values. Internally uses a bitmap.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteSet {
    bitmap: [u8; 32],
}

impl ByteSet {
    /// Returns the empty set {}.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// let set = ByteSet::empty();
    /// for x in 0..=255 {
    ///     assert!(!set.contains(x))
    /// }
    /// ```
    #[must_use]
    pub const fn empty() -> Self {
        Self { bitmap: [0; 32] }
    }

    /// Returns the set {0, ..., 255}.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// let set = ByteSet::universe();
    /// for x in 0..=255 {
    ///     assert!(set.contains(x))
    /// }
    /// ```
    #[must_use]
    pub const fn universe() -> Self {
        Self { bitmap: [u8::MAX; 32] }
    }

    /// Returns the set {`value`}.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// let set = ByteSet::point(149);
    /// for x in 0..=255 {
    ///     assert_eq!(x == 149, set.contains(x))
    /// }
    /// ```
    #[must_use]
    pub const fn point(value: u8) -> Self {
        let mut set = Self::empty();
        let (index, word) = encode(value);
        set.bitmap[index] = word;
        set
    }

    /// Returns the set {`from`, ..., `to`}.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// let set = ByteSet::range(83, 149);
    /// for x in 0..=255 {
    ///     assert_eq!((83..=149).contains(&x), set.contains(x))
    /// }
    /// ```
    #[must_use]
    pub const fn range(from: u8, to: u8) -> Self {
        let (from_index, a) = encode(from);
        let (to_index, b) = encode(to);

        let first_word = !(a - 1);
        let last_word  = b | (b - 1);

        let mut set = Self::empty();
        if from_index == to_index {
            set.bitmap[from_index] = first_word & last_word;
        } else {
            set.bitmap[from_index] = first_word;
            let mut i = from_index + 1;
            while i < to_index {
                set.bitmap[i] = u8::MAX;
                i += 1;
            }
            set.bitmap[to_index] = last_word;
        }
        set
    }

    /// Returns `true` if `self` is the empty set.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// assert!(ByteSet::empty().is_empty());
    /// assert!(!ByteSet::universe().is_empty());
    /// assert!(!ByteSet::point(0).is_empty());
    /// ```
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        let mut i = 0;
        while i < 32 {
            if self.bitmap[i] != 0 {
                return false
            }
            i += 1;
        }
        true
    }

    /// Returns `true` if `self` is the set {0, ..., 255}.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// assert!(!ByteSet::empty().is_universe());
    /// assert!(ByteSet::universe().is_universe());
    /// assert!(!ByteSet::point(0).is_universe());
    /// ```
    #[must_use]
    pub const fn is_universe(&self) -> bool {
        let mut i = 0;
        while i < 32 {
            if self.bitmap[i] == 0 {
                return false
            }
            i += 1;
        }
        true
    }

    /// Returns the smallest value in `self`. If set is empty, `None` is
    /// returned.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// assert_eq!(Some(83), ByteSet::range(83, 149).smallest());
    /// assert_eq!(None, ByteSet::empty().smallest());
    /// ```
    #[must_use]
    pub const fn smallest(&self) -> Option<u8> {
        if let Some((index, word)) = self.first() {
            Some(decode(index, word))
        } else {
            None
        }
    }

    /// Returns `true` if `value` is contained in `self`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// let set = ByteSet::range(10, 20);
    /// assert!(set.contains(15));
    /// assert!(!set.contains(25));
    /// ```
    #[must_use]
    pub const fn contains(&self, value: u8) -> bool {
        let (index, word) = encode(value);
        self.bitmap[index] & word != 0
    }

    /// Returns the complement of `self`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// let set = ByteSet::range(83, 149).complement();
    /// for x in 0..=255 {
    ///     assert_eq!(!(83..=149).contains(&x), set.contains(x))
    /// }
    /// ```
    #[must_use]
    pub const fn complement(&self) -> Self {
        let mut set = Self::empty();
        let mut i = 0;
        while i < 32 {
            set.bitmap[i] = !self.bitmap[i];
            i += 1;
        }
        set
    }

    /// Returns the intersection of `self` and `other`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// let set1 = ByteSet::range(83, 149).intersection(&ByteSet::range(59, 113));
    /// let set2 = ByteSet::range(0, 127).intersection(&ByteSet::range(128, 255));
    /// for x in 0..=255 {
    ///     assert_eq!((83..=113).contains(&x), set1.contains(x));
    /// }
    /// assert!(set2.is_empty());
    /// ```
    #[must_use]
    pub const fn intersection(&self, other: &Self) -> Self {
        let mut set = Self::empty();
        let mut i = 0;
        while i < 32 {
            set.bitmap[i] = self.bitmap[i] & other.bitmap[i];
            i += 1;
        }
        set
    }

    /// Returns the union of `self` and `other`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// let set1 = ByteSet::range(83, 149).union(&ByteSet::range(59, 113));
    /// let set2 = ByteSet::range(0, 127).union(&ByteSet::range(128, 255));
    /// for x in 0..=255 {
    ///     assert_eq!((59..=149).contains(&x), set1.contains(x));
    /// }
    /// assert!(set2.is_universe());
    /// ```
    #[must_use]
    pub const fn union(&self, other: &Self) -> Self {
        let mut set = Self::empty();
        let mut i = 0;
        while i < 32 {
            set.bitmap[i] = self.bitmap[i] | other.bitmap[i];
            i += 1;
        }
        set
    }

    /// Returns an iterator over all values in `self`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use regex_deriv::ByteSet;
    /// for (x, y) in ByteSet::range(83, 149).bytes().zip(83..=149) {
    ///     assert_eq!(y, x);
    /// }
    /// ```
    #[must_use]
    pub const fn bytes(&self) -> Bytes {
        Bytes::new(self)
    }

    /// Returns encoding of first char in set.
    #[must_use]
    const fn first(&self) -> Option<(usize, u8)> {
        let mut i = 0;
        while i < 32 {
            if self.bitmap[i] != 0 {
                return Some((i, self.bitmap[i]))
            }
            i += 1;
        }
        None
    }
}

impl std::fmt::Debug for ByteSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for word in self.bitmap {
            f.write_str(&format!("{:#010b} ", word)).unwrap();
        }
        Ok(())
    }
}

pub struct Bytes<'a> {
    set: &'a ByteSet,
    index: usize,
    word: u8,
}

impl<'a> Bytes<'a> {
    const fn new(set: &'a ByteSet) -> Self {
        if let Some((index, word)) = set.first() {
            Self { set, index, word }
        } else {
            Self { set, index: set.bitmap.len(), word: 0 }
        }
    }
}

impl Iterator for Bytes<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.word == 0 {
            if self.index < 32 - 1 {
                self.index += 1;
                self.word = self.set.bitmap[self.index];
                self.next()
            } else {
                None
            }
        } else {
            let retval = decode(self.index, self.word);
            self.word &= self.word - 1;
            Some(retval)
        }
    }
}
 
const fn encode(value: u8) -> (usize, u8) {
    let x = value as usize;
    (x / 8, 1 << (x % 8))
}

#[allow(clippy::cast_possible_truncation)]
const fn decode(index: usize, word: u8) -> u8 {
    let index = index as u8; // non-truncating as index < 32
    let trailing = word.trailing_zeros() as u8; // non-truncating as trailing <= 8
    8 * index + trailing
}

#[cfg(test)]
mod tests {
    use super::ByteSet;

    #[test]
    fn contains() {
        let set1 = ByteSet::range(10, 20);
        let set2 = ByteSet::range(30, 40);
        let set3 = ByteSet::range(50, 60);
        let set4 = ByteSet::range(70, 80);
        let set5 = ByteSet::range(90, 100);
        let set = set1.union(&set2).union(&set3).union(&set4).union(&set5);

        for x in 0..10    { assert!(!set.contains(x), "Set should not contain {:02x}", x); }
        for x in 10..=20  { assert!( set.contains(x), "{:?} Set should contain {:02x}", set, x); }
        for x in 21..30   { assert!(!set.contains(x), "Set should not contain {:02x}", x); }
        for x in 30..=40  { assert!( set.contains(x), "Set should contain {:02x}", x); }
        for x in 41..50   { assert!(!set.contains(x), "Set should not contain {:02x}", x); }
        for x in 50..=60  { assert!( set.contains(x), "Set should contain {:02x}", x); }
        for x in 61..70   { assert!(!set.contains(x), "Set should not contain {:02x}", x); }
        for x in 70..=80  { assert!( set.contains(x), "Set should contain {:02x}", x); }
        for x in 81..90   { assert!(!set.contains(x), "Set should not contain {:02x}", x); }
        for x in 90..=100 { assert!( set.contains(x), "Set should contain {:02x}", x); }
        for x in 101..110 { assert!(!set.contains(x), "Set should not contain {:02x}", x); }
    }

    #[test]
    fn intersection() {
        let set1 = ByteSet::range(60, 180);
        let set2 = set1.complement();

        assert_eq!(set2, ByteSet::range(0, 59).union(&ByteSet::range(181, 255)));
        assert_eq!(set2.intersection(&set1), ByteSet::empty());
        assert_eq!(ByteSet::empty(), ByteSet::range(0, 255).complement())
    }

    #[test]
    fn union() {
        let set1 = ByteSet::range(60, 180);
        let set2 = ByteSet::range(10, 20);
        let set3 = ByteSet::range(150, 200);

        let union = set1.union(&set2).union(&set3);

        assert_eq!(union, ByteSet::range(10, 20).union(&ByteSet::range(60, 200)));
    }

    #[test]
    fn bytes() {
        let set = ByteSet::range(1, 3).union(&ByteSet::range(5, 7));
        let mut iter = set.bytes();
        assert_eq!(iter.next(), Some(1_u8));
        assert_eq!(iter.next(), Some(2_u8));
        assert_eq!(iter.next(), Some(3_u8));
        assert_eq!(iter.next(), Some(5_u8));
        assert_eq!(iter.next(), Some(6_u8));
        assert_eq!(iter.next(), Some(7_u8));
        assert_eq!(iter.next(), None);
    }
}