use std::collections::HashSet;
use edbm::util::constraints::ClockIndex;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClockReductionInstruction {
    RemoveClock {
        clock_index: ClockIndex,
    },
    ReplaceClocks {
        clock_index: ClockIndex,
        clock_indices: HashSet<ClockIndex>,
    },
}

impl ClockReductionInstruction {
    pub(crate) fn clocks_removed_count(&self) -> usize {
        match self {
            ClockReductionInstruction::RemoveClock { .. } => 1,
            ClockReductionInstruction::ReplaceClocks { clock_indices, .. } => clock_indices.len(),
        }
    }

    pub(crate) fn get_clock_index(&self) -> ClockIndex {
        match self {
            ClockReductionInstruction::RemoveClock { clock_index }
            | ClockReductionInstruction::ReplaceClocks { clock_index, .. } => *clock_index,
        }
    }
}