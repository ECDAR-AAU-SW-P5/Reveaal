mod clock_analysis_graph;
pub mod clock_removal;
/// Module for removing clocks considered unnecessary without modifying the system.
///
/// Passive Clock Examples:
///  - Clock declared, but not used. -> Clock removed
///  - todo Clock read but never reset -> Clock set to global clock
///  - todo 2 clocks always reset at the same time -> Clocks combined into 1 clock
/// Active Clock Examples:
///  - Todo: 2 clocks never used at the same time -> Clocks combined into 1 clock
pub mod reduction;
