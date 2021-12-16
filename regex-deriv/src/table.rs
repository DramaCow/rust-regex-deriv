use super::{RegEx, DFA};

pub trait LexTable {
    const START_STATE: usize = 0;
    fn step(&self, state: usize, symbol: u8) -> usize;
    fn class(&self, state: usize) -> Option<usize>;
    fn sink(&self) -> usize;
}

pub struct NaiveLexTable {
    pub(crate) next:     Vec<usize>,
    pub(crate) classes:  Vec<Option<usize>>,
}

impl NaiveLexTable {
    #[must_use]
    pub fn new(dfa: &DFA) -> Self {      
        let nrows = dfa.states().len() - 1; // excluding sink
        let mut next = vec![nrows; 256 * nrows];
        for (i, state) in dfa.states().iter().skip(1).enumerate() {
            for (&symbol, &dest) in &state.next {
                next[256 * i + symbol as usize] = dest - 1;
            }
        }
        
        let classes = dfa.states().iter().skip(1)
            .map(|state| state.class)
            .chain(vec![None]) // <-- sink states class
            .collect();
        
        Self {
            next,
            classes,
        }
    }
}

impl LexTable for NaiveLexTable {
    fn step(&self, state: usize, symbol: u8) -> usize {
        self.next[256 * state + symbol as usize]
    }

    fn class(&self, state: usize) -> Option<usize> {
        self.classes[state]
    }

    fn sink(&self) -> usize { 
        self.classes.len() - 1
    }
}