use edbm::util::constraints::ClockIndex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    pub fn remove_clocks(&mut self, clocks_to_be_removed: &[ClockIndex]) {
        let mut clock_count = *self.clocks.values().next().unwrap_or(&(1usize));
        let mut new_clocks: HashMap<String, ClockIndex> = HashMap::new();

        for (name, _) in self
            .clocks
            .iter()
            .filter(|(_, c)| !clocks_to_be_removed.contains(c))
        {
            new_clocks.insert(name.clone(), clock_count);
            clock_count += 1;
        }

        self.clocks = new_clocks;
    }

    pub fn replace_clocks(&mut self, clock_replacements: &HashMap<ClockIndex, ClockIndex>) {
        for (clock_to_be_replaced, new_clock) in clock_replacements {
            for (_, clock) in self.clocks.iter_mut() {
                if clock == clock_to_be_replaced {
                    *clock = *new_clock;
                    break;
                }
            }
        }
    }
}
