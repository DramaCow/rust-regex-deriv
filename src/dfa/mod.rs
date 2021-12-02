use std::collections::{HashSet, HashMap, BTreeMap};
use std::iter::once;

use super::{ByteSet, RegEx, Operator};

pub struct DFA {
    states: Vec<State>,
}

pub struct State {
    pub class: Option<usize>,
    pub next: HashMap<u8, usize>,
}

impl From<&RegEx> for DFA {
    fn from(regex: &RegEx) -> Self {
        DFABuilder::build(&RegExVec::new(vec![regex.clone()]))
    }
}

impl<'a, T> From<T> for DFA
where
    T: IntoIterator<Item = &'a RegEx>,
{
    fn from(regexes: T) -> Self {
        DFABuilder::build(&RegExVec::new(regexes.into_iter().cloned().collect()))
    }
}

impl DFA {
    /// Constructs the equivalent, minimized DFA via Hopcroft's algorithm.
    #[must_use]
    pub fn minimize(&self) -> Self {
        hopcroft::minimize(self)
    }

    #[must_use]
    pub fn matches(&self, text: &str) -> bool {
        // Note: start index is always 1.
        self.class(text.bytes().fold(1, |id, byte| { self.step(id, byte) })).is_some()
    }

    #[must_use]
    pub fn step(&self, id: usize, symbol: u8) -> usize {
        match self.states[id].next.get(&symbol) {
            Some(&next_id) => next_id,
            None => 0,
        }
    }

    #[must_use]
    pub fn class(&self, id: usize) -> Option<usize> {
        self.states[id].class
    }

    #[must_use]
    pub fn states(&self) -> &[State] {
        &self.states
    }
}

// =================
// === INTERNALS ===
// =================

impl State {
    fn new(next: HashMap<u8, usize>, class: Option<usize>) -> Self {
        Self {
            class,
            next,
        }
    }

    fn sink() -> Self {
        Self::new(HashMap::new(), None)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RegExVec(Vec<RegEx>);

impl RegExVec {
    fn new(nodes: Vec<RegEx>) -> Self {
        Self(nodes)
    }

    fn sink(size: usize) -> Self {
        let none = RegEx::none();
        Self(vec![none; size])
    }

    fn deriv(&self, a: u8) -> RegExVec {
        Self(self.0.iter().map(|node| node.deriv(a)).collect())
    }

    fn class(&self) -> Option<usize> {
        self.0.iter().position(RegEx::is_nullable)
    }
}

struct DFABuilder {
    states: Vec<State>,
    re2idx: BTreeMap<RegExVec, usize>,
}

impl DFABuilder {
    fn build(start: &RegExVec) -> DFA {
        // s0 = sink state
        let states = vec![State::sink()];
        let re2idx = once((RegExVec::sink(start.0.len()), 0_usize)).collect();
        
        let mut builder = Self { states, re2idx };
        
        // s1 = start state
        builder.add_state(start);
        
        builder.explore(start, 1);

        DFA {
            states: builder.states,
        }
    }

    fn add_state(&mut self, q: &RegExVec) -> usize {
        let idx = self.states.len();
        self.re2idx.insert(q.clone(), idx);
        self.states.push(State::new(HashMap::new(), q.class()));
        idx
    }

    fn explore(&mut self, q: &RegExVec, i: usize) {
        for set in approx_deriv_classes_vec(q) {
            self.goto(q, i, &set);
        }
    }

    fn goto(&mut self, q: &RegExVec, i: usize, set: &ByteSet) {
        let c = set.smallest().unwrap();
        let qc = &q.deriv(c);

        if let Some(&j) = self.re2idx.get(qc) {
            for a in set.bytes() {
                self.states[i].next.insert(a, j);
            }
        } else {
            let j = self.add_state(qc);
            for a in set.bytes() {
                self.states[i].next.insert(a, j);
            }
            self.explore(qc, j);
        }
    }
}

fn cross<'a, B: IntoIterator<Item = &'a ByteSet>>(set1: &HashSet<ByteSet>, set2: B) -> HashSet<ByteSet> {
    set2.into_iter().flat_map(|t| {
        set1.iter().filter_map(move |s| {
            let u = t.intersection(&s);
            if u.is_empty() { None } else { Some(u) }
        })
    }).collect()
}

// TODO: memoize
fn approx_deriv_classes(root: &RegEx) -> HashSet<ByteSet> {
    let mut stack = vec![root];
    let mut charsets: HashSet<ByteSet> = once(ByteSet::universe()).collect();
    
    while let Some(node) = stack.pop() {
        match node.operator() {
            Operator::None | Operator::Epsilon => {
                // Do nothing. C(eps) = {universe}, so C(r) ^ C(eps) = C(r).
            },
            Operator::Set(set) => {
                // TODO: set cannot be empty?
                if !set.is_empty() && !set.is_universe() {
                    let cset = &set.complement();
                    charsets = cross(&charsets, vec![set, cset].into_iter());
                }
            },
            Operator::Cat(children) => {
                let mut tail = &children[..];
                while let Some(head) = tail.first() {
                    stack.push(head);
                    if !head.is_nullable() {
                        break;
                    }
                    tail = &tail[1..];
                }
            },
            Operator::Star(child) | Operator::Not(child) => {
                stack.push(child);
            },
            Operator::Or(children) | Operator::And(children) => {
                for child in children {
                    stack.push(child);
                }
            },
        }
    }
    
    charsets
}

fn approx_deriv_classes_vec(root: &RegExVec) -> HashSet<ByteSet> {
    root.0.iter().fold(once(ByteSet::universe()).collect(), |acc, x| {
        cross(&acc, &approx_deriv_classes(x))
    })
}

mod hopcroft;

#[cfg(test)]
mod tests;