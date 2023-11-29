use edbm::util::constraints::ClockIndex;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClockReductionInstruction {
    RemoveClock {
        clock_index: ClockIndex,
    },
    ReplaceClock {
        clock_index: ClockIndex,
        replacing_clock: ClockIndex,
    },
}

impl ClockReductionInstruction {
    pub fn get_clock_index(&self) -> ClockIndex {
        match self {
            ClockReductionInstruction::RemoveClock { clock_index, .. }
            | ClockReductionInstruction::ReplaceClock { clock_index, .. } => *clock_index,
        }
    }
}
