use std::collections::{BTreeSet, HashSet, HashMap, hash_map::Entry};
use super::{DFA, State};

pub fn minimize(dfa: &DFA) -> DFA {
    // Since the accept state must always be reachable by some input string
    // from the start state in any well-formed DFA, the equivalence class
    // containing the start state must be distinct from the equivalence
    // class containing the sink state. Hence, a minimised DFA must have
    // a distinct start and sink state.

    // Thanks to lexicographical ordering of BTreeSet elements,
    // the first set in the partition is guaranteed to be the sink class
    // and the second set is guaranteed to be the start class.
    let partition = equivalence_classes(dfa);

    let mut states = Vec::with_capacity(partition.len());
    states.push(State::sink());

    for set in partition.iter().skip(1) {
        let mut next = HashMap::new();

        for &source in set {
            for (&symbol, dest) in &dfa.states[source].next {
                if let Entry::Vacant(e) = next.entry(symbol) {
                    if let Some(id) = partition.iter().skip(1).position(|set| set.contains(&dest)) {
                        e.insert(id + 1);
                    }
                }
            }
        }

        states.push(State::new(next, set.iter().filter_map(|&id| dfa.states[id].class).min()));
    }

    DFA { states }
}

// =================
// === INTERNALS ===
// =================

type Ids = BTreeSet<usize>;
type Partition = BTreeSet<Ids>;

/// Partitions the states of a well-formed input `DFA` such that
/// each set of the partition is an equivalence class of the
/// Myhill-Nerode equivalence relation (i.e. starting from either
/// state, all remaining substrings will result in the same accept
/// or reject behaviour)
fn equivalence_classes(dfa: &DFA) -> Partition {
    // while the alphabet technically consists of all possible chars,
    // we are only required to consider those that appear in transitions
    // for this DFA.
    let alph = alphabet(&dfa.states); // O(kn)

    // precompute inverse of transition function
    let idfa = InvDFA::new(&dfa, &alph); // O(kn)
    
    let mut partition = coarse_partition(&dfa); // O(n)
    let mut waiting   = all_but_largest(&partition);

    while !waiting.is_empty() {
        let w = take_some(&mut waiting);

        // where inv is the domain that maps to w via symbol. TODO: We don't have to consider all chars in alph,
        // in fact we could just consider the chars that are used to access states in w from inv; investigate.
        for inv in alph.iter().filter_map(|&symbol| idfa.inverse(&w, symbol)) {

            // all sets of the current partition that are split by symbol 
            // (i.e. that have a non-empty subset mapping to w via symbol and
            // a non-empty subset not mapping to w via symbol)
            let splits = split_sets(&partition, &inv).collect::<Vec<_>>();

            for (p, (p1, p2)) in splits {
                partition.remove(&p);
                
                if waiting.remove(&p) {
                    waiting.insert(p1.clone());
                    waiting.insert(p2.clone());
                } else if p1.len() <= p2.len() {
                    waiting.insert(p1.clone());
                } else {
                    waiting.insert(p2.clone());
                }
                
                partition.insert(p1);
                partition.insert(p2);
            }
        }
    }

    partition
}

/// Represents the inverse of the transition function;
/// that is, for a given state q and char c, stores
/// the set of states that transition to q via c. 
struct InvDFA {
    states: Vec<HashMap<u8, Ids>>,
}

impl InvDFA {
    fn new(dfa: &DFA, alph: &[u8]) -> Self {
        let mut states = vec![HashMap::<_, Ids>::new(); dfa.states.len()];
        for (source_id, state) in dfa.states.iter().enumerate() {
            // for (&symbol, &dest_id) in &state.next {
            for &symbol in alph {
                let dest_id = if let Some(&dest_id) = state.next.get(&symbol) { dest_id } else { 0 };
                states[dest_id].entry(symbol).or_default().insert(source_id);
            }
        }
        Self { states }
    }

    /// Returns the union of the inverse of the transition function for
    /// all states in ids via character symbol.
    fn inverse(&self, ids: &Ids, symbol: u8) -> Option<Ids> {
        let mut iter = ids.iter().filter_map(|&id| self.states[id].get(&symbol));
        let set = iter.next().cloned()?;
        Some(iter.fold(set, |mut set, keys| {
            set.extend(keys); set
        }))
    }
}

fn alphabet(states: &[State]) -> Vec<u8> {
    let mut iter = states.iter().map(|state| state.next.keys());
    let alph = iter.next().unwrap().copied().collect::<HashSet<_>>();
    let alph = iter.fold(alph, |mut alph, keys| {
        alph.extend(keys); alph
    });
    alph.into_iter().collect() // TODO: should we bother copying into vector?
}

/// Produces an initial partition of states such that all pairs of 
/// Myhill-Nerode equivalent nodes are in the same set. However, this
/// initial partitioning is coarse; the sets of the partition may contain
/// pairs of nodes that aren't equivalent. The goal of the Hopcroft
/// algorithm is to repeatedly refine the paritioning such that each set
/// strictly contains equivalent nodes.
fn coarse_partition(dfa: &DFA) -> Partition {
    let mut partition: HashMap<_, Ids> = HashMap::new();
    for (id, state) in dfa.states.iter().enumerate() {
        let class = state.class.map_or(0, |class| class + 1);
        partition.entry(class).or_default().insert(id);
    }
    partition.values().cloned().collect()
}

fn all_but_largest(partition: &Partition) -> BTreeSet<Ids> {
    let argmax = partition.iter().enumerate().max_by_key(|(_, set)| set.len()).map(|(i, _)| i).unwrap();
    partition.iter().enumerate().filter_map(|(i, set)| if i == argmax { None } else { Some(set) }).cloned().collect()
}

fn take_some(waiting: &mut BTreeSet<Ids>) -> Ids {
    waiting.take(&waiting.iter().next().cloned().unwrap()).unwrap()
}

fn split_sets<'a>(sets: &'a Partition, domain: &'a Ids) -> impl Iterator<Item=(Ids, (Ids, Ids))> + 'a {
    sets.iter().filter_map(move |p| {
        let inter: Ids = p.intersection(domain).copied().collect();

        if inter.is_empty() {
            None
        } else {
            let diff: Ids = p.difference(domain).copied().collect();
            if diff.is_empty() {
                None
            } else {
                Some((p.clone(), (inter, diff)))
            }
        }
    })
}