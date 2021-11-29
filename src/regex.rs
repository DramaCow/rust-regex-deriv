#![allow(clippy::match_same_arms)]

use std::rc::Rc;
use std::iter::once;
use std::fmt::Formatter;
use std::fmt::Error;
use std::fmt::Debug;

use itertools::Itertools;
use super::CharSet;

/// Regular expression object. Internally, represented by an
/// expression tree.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegEx {
    root: Rc<Operator>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Operator {
    None,
    Epsilon,

    /// # Invariants
    /// * Set is not empty
    Set(CharSet),

    /// # Invariants
    /// * At least 2 children
    /// * No child is None
    /// * No child is Epsilon
    /// * No child is Cat
    Cat(Vec<RegEx>),

    /// # Invariants
    /// * Child is not None
    /// * Child is not Epsilon
    /// * Child is not Star
    Star(RegEx),

    /// # Invariants
    /// * At least 2 children
    /// * No child is None
    /// * No child is Or
    /// * At most 1 child is a Set
    Or(Vec<RegEx>),

    /// # Invariants
    /// * At least 2 children
    /// * No child is None
    /// * No child is Epsilon
    /// * No child is And
    /// * At most 1 child is a Set
    And(Vec<RegEx>),

    /// # Invariants
    /// * Child is not None
    /// * Child is not a Set
    /// * Child is not Not
    Not(RegEx),
}

impl RegEx {
    // === canonical constructors ===

    #[must_use]
    pub fn none() -> Self {
        Self::new(Operator::None)
    }

    #[must_use]
    pub fn empty() -> Self {
        Self::new(Operator::Epsilon)
    }

    #[must_use]
    pub fn set(a: CharSet) -> Self {
        fn mk_set(a: CharSet) -> RegEx {
            if a.is_empty() {
                RegEx::new(Operator::None)
            } else {
                RegEx::new(Operator::Set(a))
            }
        }

        mk_set(a)
    }

    #[must_use]
    pub fn then(&self, other: &Self) -> Self {
        fn mk_cat(r: &RegEx, s: &RegEx) -> RegEx {
            fn cat_aux<'a, A, B>(res1: A, res2: B) -> RegEx
            where
                A: IntoIterator<Item=&'a RegEx>,
                B: IntoIterator<Item=&'a RegEx>,
            {
                RegEx::new(Operator::Cat(res1.into_iter().chain(res2).cloned().collect()))
            }
        
            match (r.operator(), s.operator()) {
                (_                , Operator::Epsilon) => r.clone(),
                (Operator::Epsilon, _                ) => s.clone(),
                (_                , Operator::None   ) => RegEx::new(Operator::None),
                (Operator::None   , _                ) => RegEx::new(Operator::None),
                (Operator::Cat(a) , Operator::Cat(b) ) => cat_aux(a, b),
                (_                , Operator::Cat(b) ) => cat_aux(once(r), b),
                (Operator::Cat(a) , _                ) => cat_aux(a, once(s)),
                (_                , _                ) => cat_aux(once(r), once(s)),
            }
        }

        mk_cat(self, other)
    }

    #[must_use]
    pub fn star(&self) -> Self {
        fn mk_star(r: &RegEx) -> RegEx {
            match *r.root {
                Operator::None | Operator::Epsilon => RegEx::new(Operator::Epsilon),
                Operator::Star(_)                  => r.clone(),
                _                                  => RegEx::new(Operator::Star(r.clone())),
            }
        }

        mk_star(self)
    }

    #[must_use]
    pub fn or(&self, other: &Self) -> Self {
        fn mk_or(r: &RegEx, s: &RegEx) -> RegEx {
            fn or_aux<'a, A, B>(res1: A, res2: B) -> RegEx
            where
                A: IntoIterator<Item=&'a RegEx>,
                B: IntoIterator<Item=&'a RegEx>,
            {
                let refs = merged_sets(res1.into_iter().merge(res2), |mut a, b| { a.union_assign(b); a });
        
                if refs.is_empty() {
                    RegEx::new(Operator::None)
                } else if refs.len() == 1 {
                    refs[0].clone()
                } else {
                    RegEx::new(Operator::Or(refs))
                }
            }
        
            match (r.operator(), s.operator()) {
                (_               , Operator::None  ) => r.clone(),
                (Operator::None  , _               ) => s.clone(),
                (Operator::Set(x), Operator::Set(y)) => RegEx::set(x.union(&y)),
                (Operator::Or(a) , Operator::Or(b) ) => or_aux(a, b),
                (Operator::Or(a) , _               ) => or_aux(a, once(s)),
                (_               , Operator::Or(b) ) => or_aux(once(r), b),
                (_               , _               ) => or_aux(once(r), once(s)),
            }
        }

        mk_or(self, other)
    }

    #[must_use]
    pub fn and(&self, other: &Self) -> Self {
        fn mk_and(r: &RegEx, s: &RegEx) -> RegEx {
            fn and_aux<'a, A, B>(res1: A, res2: B) -> RegEx
            where
                A: IntoIterator<Item=&'a RegEx>,
                B: IntoIterator<Item=&'a RegEx>,
            {
                let refs = merged_sets(res1.into_iter().merge(res2), |mut a, b| { a.intersection_assign(b); a });
        
                if refs.is_empty() {
                    RegEx::new(Operator::None)
                } else if refs.len() == 1 {
                    refs[0].clone()
                } else {
                    RegEx::new(Operator::And(refs))
                }
            }
        
            match (r.operator(), s.operator()) {
                (_                , Operator::None   ) => RegEx::new(Operator::None),
                (Operator::None   , _                ) => RegEx::new(Operator::None),
                (_                , Operator::Epsilon) => if r.is_nullable() { RegEx::new(Operator::Epsilon) } else { RegEx::new(Operator::None) }, // TODO: check
                (Operator::Epsilon, _                ) => if s.is_nullable() { RegEx::new(Operator::Epsilon) } else { RegEx::new(Operator::None) }, // TODO: check
                (Operator::Set(x) , Operator::Set(y) ) => RegEx::set(x.intersection(&y)),
                (Operator::And(a) , Operator::And(b) ) => and_aux(a, b),
                (Operator::And(a) , _                ) => and_aux(a, once(s)),
                (_                , Operator::And(b) ) => and_aux(once(r), b),
                (_                , _                ) => and_aux(once(r), once(s)),
            }
        }

        mk_and(self, other)
    }

    #[must_use]
    pub fn not(&self) -> Self {
        fn mk_not(r: &RegEx) -> RegEx {
            match r.operator() {
                Operator::None   => RegEx::set(CharSet::universe()),
                Operator::Set(s) => RegEx::set(s.complement()),
                Operator::Not(a) => a.clone(),
                _                => RegEx::new(Operator::Not(r.clone())),
            }
        }

        mk_not(self)
    }

    // === non-canonical constructors ===

    #[must_use]
    pub fn opt(&self) -> Self {
        self.or(&RegEx::empty())
    }

    #[must_use]
    pub fn plus(&self) -> Self {
        self.then(&self.star())
    }

    #[must_use]
    pub fn diff(&self, other: &Self) -> Self {
        self.and(&other.not())
    }

    // === other functions ===

    #[must_use]
    pub fn deriv(&self, a: u8) -> Self {
        fn deriv_cat(children: &[RegEx], a: u8) -> RegEx {
            fn aux(r: &RegEx, s: &RegEx, a: u8) -> RegEx {
                let nu_r_da_s = if r.is_nullable() {
                    s.deriv(a)
                } else {
                    RegEx::new(Operator::None)
                };
                (r.deriv(a).then(s)).or(&nu_r_da_s)
            }
    
            match children {
                [] | [_] => {
                    unreachable!("Should be impossible for Cat node to have <2 children.")
                },
                [r, s] => {
                    aux(r, s, a)
                },
                [r, ..] => {
                    // Tail of children still form a valid Cat node.
                    let s = &RegEx::new(Operator::Cat(children[1..].to_vec()));
                    aux(r, s, a)
                },
            }
        }
        
        fn deriv_or(children: &[RegEx], a: u8) -> RegEx {
            match children {
                [] | [_] => {
                    unreachable!("Should be impossible for Or node to have <2 children.")
                },
                [r, s] => {
                    r.deriv(a).or(&s.deriv(a))
                },
                [r, ..] => {
                    let s = &RegEx::new(Operator::Or(children[1..].to_vec()));
                    r.deriv(a).or(&s.deriv(a))
                },
            }
        }
        
        fn deriv_and(children: &[RegEx], a: u8) -> RegEx {
            match children {
                [] | [_] => {
                    unreachable!("Should be impossible for And node to have <2 children.")
                },
                [r, s] => {
                    r.deriv(a).and(&s.deriv(a))
                },
                [r, ..] => {
                    let s = &RegEx::new(Operator::And(children[1..].to_vec()));
                    r.deriv(a).and(&s.deriv(a))
                },
            }
        }
    
        match self.operator() {
            Operator::None
            | Operator::Epsilon => RegEx::new(Operator::None),
            Operator::Set(s)    => if s.contains(a) { RegEx::new(Operator::Epsilon) } else { RegEx::new(Operator::None) },
            Operator::Cat(res)  => deriv_cat(res, a),
            Operator::Star(re)  => re.deriv(a).then(self),
            Operator::Or(res)   => deriv_or(res, a),
            Operator::And(res)  => deriv_and(res, a),
            Operator::Not(re)   => re.deriv(a).not(),
        }
    }

    #[must_use]
    pub fn operator(&self) -> &Operator {
        &*self.root
    }

    /// Returns true iff recognizes epsilon.
    #[must_use]
    pub fn is_nullable(&self) -> bool {
        match self.operator() {
            Operator::None     => false,
            Operator::Epsilon  => true,
            Operator::Set(_)   => false,
            Operator::Cat(res) => res.iter().all(RegEx::is_nullable),
            Operator::Star(_)  => true,
            Operator::Or(res)  => res.iter().any(RegEx::is_nullable),
            Operator::And(res) => res.iter().all(RegEx::is_nullable),
            Operator::Not(re)  => !re.is_nullable(),
        }
    }
}

// =================
// === INTERNALS ===
// =================

impl RegEx {
    fn new(node: Operator) -> RegEx {
        RegEx { root: Rc::new(node) }
    }
}

// TODO: improve
fn merged_sets<'a, T, F>(res: T, f: F) -> Vec<RegEx>
where
    T: IntoIterator<Item=&'a RegEx>,
    F: Fn(CharSet, &CharSet) -> CharSet,
{
    let mut sets: Vec<&CharSet> = Vec::new();
    let mut refs: Vec<&RegEx> = Vec::new();

    for re in res {
        if let Operator::Set(a) = re.operator() {
            sets.push(a);
        } else {
            refs.push(re);
        }
    }

    #[allow(clippy::option_if_let_else)]
    if let Some(last) = sets.pop() {
        let reduced_set = sets.into_iter().fold(last.clone(), |acc, x| f(acc, x));
        let re = RegEx::new(Operator::Set(reduced_set));
        refs.into_iter().merge(once(&re)).cloned().collect()
    } else {
        refs.into_iter().cloned().collect()
    }
}

impl Debug for Operator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Operator::None => {
                f.write_str("\u{2205}")
            },
            Operator::Epsilon => {
                f.write_str("\u{03B5}")
            },
            Operator::Set(set) => {
                f.write_str(&format!("{:?}", set))
            },
            Operator::Cat(children) => {
                f.write_str(&format!("({})", children.iter().map(|child| format!("{:?}", child)).collect::<String>()))
            },
            Operator::Star(child) => {
                f.write_str(&format!("({:?})*", child))
            },
            Operator::Or(children) => {
                f.write_str(&format!("({})", children.iter().map(|child| format!("{:?}", child)).collect::<Vec<_>>().join("|")))

            },
            Operator::And(children) => {
                f.write_str(&format!("({})", children.iter().map(|child| format!("{:?}", child)).collect::<Vec<_>>().join("&")))

            },
            Operator::Not(child) => {
                f.write_str(&format!("!({:?})", child))
            },
        }
    }
}

impl Debug for RegEx {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(&format!("{:?}", *self.root))
    }
}