use edbm::util::constraints::ClockIndex;
use serde::{Deserialize, Serialize};
use std::collections::hash_set::Iter;
use std::collections::{HashMap, HashSet};

/// The declaration struct is used to hold the indices for each clock, and is meant to be the owner of int variables once implemented
#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Serialize)]
pub struct Declarations {
    pub ints: HashMap<String, i32>,
    pub clocks: HashMap<String, ClockIndex>,
}

pub trait DeclarationProvider {
    fn get_declarations(&self) -> &Declarations;
}

impl Declarations {
    pub fn empty() -> Declarations {
        Declarations {
            ints: HashMap::new(),
            clocks: HashMap::new(),
        }
    }

    pub fn get_clock_count(&self) -> usize {
        self.clocks.len()
    }

    pub fn set_clock_indices(&mut self, start_index: ClockIndex) {
        for (_, v) in self.clocks.iter_mut() {
            *v += start_index
        }
    }

    pub fn get_clock_index_by_name(&self, name: &str) -> Option<&ClockIndex> {
        self.clocks.get(name)
    }

    /// Gets the name of a given `ClockIndex`.
    /// Returns `None` if it does not exist in the declarations
    pub fn get_clock_name_by_index(&self, index: ClockIndex) -> Option<&String> {
        self.clocks
            .iter()
            .find(|(_, v)| **v == index)
            .map(|(k, _)| k)
    }

    pub fn remove_clocks(&mut self, clocks_to_be_removed: &Vec<ClockIndex>) {
        let mut new_clocks: HashMap<String, ClockIndex> = HashMap::new();

        for (name, old_clock) in self
            .clocks
            .iter()
            .filter(|(_, c)| !clocks_to_be_removed.contains(c))
        {
            let clocks_less = clocks_to_be_removed.partition_point(|clock| clock < old_clock);
            new_clocks.insert(name.clone(), *old_clock - clocks_less);
        }

        self.clocks = new_clocks;
    }

    pub fn combine_clocks(&mut self, combine_clocks: &Vec<HashSet<ClockIndex>>) {
        for clock_group in combine_clocks {
            let mut clock_group_iter: Iter<ClockIndex> = clock_group.iter();
            let first_clock = match clock_group_iter.next() {
                None => {
                    continue;
                }
                Some(clock) => clock,
            };
            let mut next_clock_to_be_combined = match clock_group_iter.next() {
                None => {
                    continue;
                }
                Some(clock) => clock,
            };
            for (_, current_clock) in self.clocks.iter_mut() {
                if current_clock == next_clock_to_be_combined {
                    *current_clock = *first_clock;
                    next_clock_to_be_combined = match clock_group_iter.next() {
                        None => {
                            break;
                        }
                        Some(clock) => clock,
                    };
                }
            }
        }
    }
}
