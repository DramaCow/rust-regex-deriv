use super::{RegEx, DFA};

#[derive(Clone, Copy)]
pub enum Command {
    Skip,
    Emit,
}

pub trait LexTable {
    const START_STATE: usize = 0;
    fn step(&self, state: usize, symbol: u8) -> usize;
    fn class(&self, state: usize) -> Option<usize>;
    fn sink(&self) -> usize;
    fn command(&self, class: usize) -> Command;
}

pub struct NaiveLexTable {
    pub(crate) next:     Vec<usize>,
    pub(crate) classes:  Vec<Option<usize>>,
    pub(crate) commands: Vec<Command>,
}

impl NaiveLexTable {
    #[must_use]
    pub fn new<'a, T, C>(regexes: T, commands: C) -> Self
    where
        T: IntoIterator<Item = &'a RegEx>,
        C: IntoIterator<Item = Command>,
    {
        let dfa = DFA::from(regexes).minimize();
        
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
            commands: commands.into_iter().collect(),
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

    fn command(&self, class: usize) -> Command {
        self.commands[class]
    }
}