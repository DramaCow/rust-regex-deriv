#![allow(clippy::needless_range_loop)]

type Word = u32; // type used for bitmap

const NUM_WORDS: usize = 256 / Word::BITS as usize;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharSet {
    bitmap: [Word; NUM_WORDS],
}

impl CharSet {
    #[must_use]
    pub fn empty() -> Self {
        Self { bitmap: [0; NUM_WORDS] }
    }

    #[must_use]
    pub fn universe() -> Self {
        Self { bitmap: [Word::MAX; NUM_WORDS] }
    }

    #[must_use]
    fn words(&self) -> &[Word] {
        &self.bitmap
    }

    #[must_use]
    pub fn point(value: u8) -> Self {
        let mut set = Self::empty();
        let (index, word) = encode(value);
        set.bitmap[index] = word;
        set
    }

    #[must_use]
    pub fn range(from: u8, to: u8) -> Self {
        let (from_index, a) = encode(from);
        let (to_index, b) = encode(to);

        let first_word = !(a - 1);
        let last_word  = b | (b - 1);

        let mut set = Self::empty();
        if from_index == to_index {
            set.bitmap[from_index] = first_word & last_word;
        } else {
            set.bitmap[from_index] = first_word;
            for i in from_index+1..to_index {
                set.bitmap[i] = Word::MAX;
            }
            set.bitmap[to_index] = last_word;
        }
        set
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bitmap.iter().copied().all(|byte| byte == 0)
    }

    #[must_use]
    pub fn is_universe(&self) -> bool {
        self.bitmap.iter().copied().all(|byte| byte == Word::MAX)
    }

    #[must_use]
    pub fn min(&self) -> Option<u8> {
        let (index, word) = self.first_word()?;
        Some(decode(index, word))
    }

    #[must_use]
    pub fn contains(&self, x: u8) -> bool {
        let (index, word) = encode(x);
        self.bitmap[index] & word != 0
    }

    #[must_use]
    pub fn complement(&self) -> Self {
        let mut set = Self::empty();
        for i in 0..NUM_WORDS {
            set.bitmap[i] = !self.bitmap[i]
        }
        set
    }

    #[must_use]
    pub fn intersection(&self, other: &Self) -> Self {
        let mut set = Self::empty();
        for i in 0..NUM_WORDS {
            set.bitmap[i] = self.bitmap[i] & other.bitmap[i]
        }
        set
    }

    pub fn intersection_assign(&mut self, other: &Self) {
        for i in 0..NUM_WORDS {
            self.bitmap[i] &= other.bitmap[i];
        }
    }

    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        let mut set = Self::empty();
        for i in 0..NUM_WORDS {
            set.bitmap[i] = self.bitmap[i] | other.bitmap[i]
        }
        set
    }

    pub fn union_assign(&mut self, other: &Self) {
        for i in 0..NUM_WORDS {
            self.bitmap[i] |= other.bitmap[i];
        }
    }

    #[must_use]
    pub fn chars(&self) -> Chars {
        Chars::new(self)
    }

    #[must_use]
    fn first_word(&self) -> Option<(usize, Word)> {
        self.words().iter().copied().enumerate().find(|&(_, word)| word != 0)
    }
}

impl std::fmt::Debug for CharSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        for word in self.words() {
            f.write_str(&format!("{:#010b} ", word)).unwrap();
        }
        Ok(())
    }
}

pub struct Chars<'a> {
    set: &'a CharSet,
    index: usize,
    word: Word,
}

impl<'a> Chars<'a> {
    fn new(set: &'a CharSet) -> Self {
        if let Some((index, word)) = set.first_word() {
            Self { set, index, word }
        } else {
            Self { set, index: set.bitmap.len(), word: 0 }
        }
    }
}

impl Iterator for Chars<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.word == 0 {
            if self.index < NUM_WORDS - 1 {
                self.index += 1;
                self.word = self.set.bitmap[self.index];
                self.next()
            } else {
                None
            }
        } else {
            let retval = decode(self.index, self.word);
            self.word &= self.word.wrapping_sub(1);
            Some(retval)
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
fn encode(value: u8) -> (usize, Word) {
    let x = value as u32;
    ((x / Word::BITS) as usize, 1 << (x % Word::BITS))
}

#[allow(clippy::cast_possible_truncation)]
fn decode(index: usize, word: Word) -> u8 {
    (Word::BITS * index as u32 + word.trailing_zeros()) as u8
}

#[cfg(test)]
mod tests {
    use super::CharSet;

    #[test]
    fn contains() {
        let set1 = CharSet::range(10, 20);
        let set2 = CharSet::range(30, 40);
        let set3 = CharSet::range(50, 60);
        let set4 = CharSet::range(70, 80);
        let set5 = CharSet::range(90, 100);
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
        let set1 = CharSet::range(60, 180);
        let set2 = set1.complement();

        assert_eq!(set2, CharSet::range(0, 59).union(&CharSet::range(181, 255)));
        assert_eq!(set2.intersection(&set1), CharSet::empty());
        assert_eq!(CharSet::empty(), CharSet::range(0, 255).complement())
    }

    #[test]
    fn union() {
        let set1 = CharSet::range(60, 180);
        let set2 = CharSet::range(10, 20);
        let set3 = CharSet::range(150, 200);

        let union = set1.union(&set2).union(&set3);

        assert_eq!(union, CharSet::range(10, 20).union(&CharSet::range(60, 200)));
    }

    #[test]
    fn chars() {
        let set = CharSet::range(1, 3).union(&CharSet::range(5, 7));
        let mut iter = set.chars();
        assert_eq!(iter.next(), Some(1_u8));
        assert_eq!(iter.next(), Some(2_u8));
        assert_eq!(iter.next(), Some(3_u8));
        assert_eq!(iter.next(), Some(5_u8));
        assert_eq!(iter.next(), Some(6_u8));
        assert_eq!(iter.next(), Some(7_u8));
        assert_eq!(iter.next(), None);
    }
}