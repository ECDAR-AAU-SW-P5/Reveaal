use crate::system::query_failures::SystemRecipeFailure;
use crate::transition_systems::clock_reduction::clock_analysis_graph::{
    ClockAnalysisEdge, ClockAnalysisGraph, ClockAnalysisNode,
};
use crate::transition_systems::clock_reduction::clock_reduction_instruction::ClockReductionInstruction;
use crate::transition_systems::{LocationTree, TransitionSystemPtr};
use edbm::util::constraints::ClockIndex;
use log::debug;
use std::collections::{HashMap, HashSet, VecDeque};

/// Function for a "safer" clock reduction that handles both the dimension of the DBM and the quotient index if needed be
/// # Arguments
/// `lhs`: The (main) [`TransitionSystemPtr`] to clock reduce\n
/// `rhs`: An optional [`TransitionSystemPtr`] used for multiple operands (Refinement)\n
/// `dim`: A mutable reference to the DBMs dimension for updating\n
/// `quotient_clock`: The clock for the quotient (This is not reduced)
/// # Returns
/// A `Result` used if the [`TransitionSystemPtr`](s) fails
pub fn clock_reduce(
    lhs: &mut TransitionSystemPtr,
    rhs: Option<&mut TransitionSystemPtr>,
    dim: &mut usize,
    quotient_clock: Option<ClockIndex>,
) -> Result<(), Box<SystemRecipeFailure>> {
    if *dim == 0 {
        return Ok(());
    } else if rhs.is_none() {
        return clock_reduce_single(lhs, dim, quotient_clock);
    }
    let rhs = rhs.unwrap();

    let (l_clocks, r_clocks) = filter_redundant_clocks(
        find_redundant_clocks(lhs),
        find_redundant_clocks(rhs),
        quotient_clock,
        lhs.get_dim(),
    );

    debug!("Clocks to be reduced: {l_clocks:?} + {r_clocks:?}");
    *dim -= l_clocks
        .iter()
        .chain(r_clocks.iter())
        .fold(0, |acc, c| acc + c.clocks_removed_count());
    debug!("New dimension: {dim}");

    let (l_remove_clocks, l_replace_clocks) = extract_remove_and_replace_from_instruction(l_clocks);
    let (r_remove_clocks, r_replace_clocks) = extract_remove_and_replace_from_instruction(r_clocks);

    lhs.remove_clocks(&l_remove_clocks).unwrap();
    rhs.remove_clocks(&r_remove_clocks).unwrap();
    lhs.replace_clocks(&l_replace_clocks).unwrap();
    rhs.replace_clocks(&r_replace_clocks).unwrap();

    Ok(())
}

/// Clock reduces a "single_expression", such as consistency
/// # Arguments
///
/// * `sys`: The [`SystemRecipe`] to clock reduce
/// * `dim`: the dimension of the system
/// * `quotient_clock`: The clock for the quotient (This is not reduced)
///
/// returns: Result<(), SystemRecipeFailure>
fn clock_reduce_single(
    sys: &mut TransitionSystemPtr,
    dim: &mut usize,
    quotient_clock: Option<ClockIndex>,
) -> Result<(), Box<SystemRecipeFailure>> {
    let mut clocks = find_redundant_clocks(sys);
    clocks.retain(|ins| ins.get_clock_index() != quotient_clock.unwrap_or_default());
    debug!("Clocks to be reduced: {clocks:?}");
    *dim -= clocks
        .iter()
        .fold(0, |acc, c| acc + c.clocks_removed_count());
    debug!("New dimension: {dim}");
    let (remove_clocks, replace_clocks) = extract_remove_and_replace_from_instruction(clocks);
    sys.remove_clocks(&remove_clocks).unwrap();
    sys.replace_clocks(&replace_clocks).unwrap();
    Ok(())
}

//todo fix clockreductioninstruction
fn extract_remove_and_replace_from_instruction(
    instructions: Vec<ClockReductionInstruction>,
) -> (Vec<ClockIndex>, HashMap<ClockIndex, ClockIndex>) {
    let mut remove_clocks: Vec<ClockIndex> = Vec::new();
    let mut replace_clocks: HashMap<ClockIndex, ClockIndex> = HashMap::new();
    for instruction in instructions {
        match instruction {
            ClockReductionInstruction::RemoveClock { clock_index } => {
                remove_clocks.push(clock_index)
            }
            ClockReductionInstruction::ReplaceClocks {
                clock_index,
                clock_indices,
            } => {
                for remove_clock in clock_indices {
                    replace_clocks.insert(remove_clock, clock_index);
                }
            }
        }
    }
    return (remove_clocks, replace_clocks);
}

fn filter_redundant_clocks(
    lhs: Vec<ClockReductionInstruction>,
    rhs: Vec<ClockReductionInstruction>,
    quotient_clock: Option<ClockIndex>,
    split_index: ClockIndex,
) -> (
    Vec<ClockReductionInstruction>,
    Vec<ClockReductionInstruction>,
) {
    fn get_unique_redundant_clocks<P: Fn(ClockIndex) -> bool>(
        l: Vec<ClockReductionInstruction>,
        r: Vec<ClockReductionInstruction>,
        quotient: ClockIndex,
        bound_predicate: P,
    ) -> Vec<ClockReductionInstruction> {
        l.into_iter()
            // Takes clock instructions that also occur in the rhs system
            // This is done because the lhs also finds the redundant clocks from the rhs,
            // so to ensure that it should be removed, we check if it occurs on both sides
            // which would mean it can be removed
            // e.g "A <= B", we can find clocks from B that are not used in A, so they are marked as remove
            .filter(|ins| r.contains(ins))
            // Takes all the clocks within the bounds of the given system
            // This is done to ensure that we don't try to remove a clock from the rhs system
            .filter(|ins| bound_predicate(ins.get_clock_index()))
            // Removes the quotient clock
            .filter(|ins| ins.get_clock_index() != quotient)
            .collect()
    }
    let quotient_clock = quotient_clock.unwrap_or_default();
    (
        get_unique_redundant_clocks(lhs.clone(), rhs.clone(), quotient_clock, |c| {
            c <= split_index
        }),
        get_unique_redundant_clocks(rhs, lhs, quotient_clock, |c| c > split_index),
    )
}

///Helper function to recursively traverse all transitions in a transitions system
///in order to find all transitions and location in the transition system, and
///saves these as [ClockAnalysisEdge]s and [ClockAnalysisNode]s in the [ClockAnalysisGraph]
pub fn find_edges_and_nodes(
    system: &TransitionSystemPtr,
    init_location: LocationTree,
    graph: &mut ClockAnalysisGraph,
) {
    let mut worklist = VecDeque::from([init_location]);
    let actions = system.get_actions();
    while let Some(location) = worklist.pop_front() {
        //Constructs a node to represent this location and add it to the graph.
        let mut node: ClockAnalysisNode = ClockAnalysisNode {
            invariant_dependencies: HashSet::new(),
            id: location.id.get_unique_string(),
        };

        //Finds clocks used in invariants in this location.
        if let Some(invariant) = &location.invariant {
            let conjunctions = invariant.minimal_constraints().conjunctions;
            for conjunction in conjunctions {
                for constraint in conjunction.iter() {
                    node.invariant_dependencies.insert(constraint.i);
                    node.invariant_dependencies.insert(constraint.j);
                }
            }
        }
        graph.nodes.insert(node.id.clone(), node);

        //Constructs an edge to represent each transition from this graph and add it to the graph.
        for action in &actions {
            for transition in system.next_transitions_if_available(&location, action) {
                let mut edge = ClockAnalysisEdge {
                    from: location.id.get_unique_string(),
                    to: transition.target_locations.id.get_unique_string(),
                    guard_dependencies: HashSet::new(),
                    updates: transition.updates,
                    edge_type: action.to_string(),
                };

                //Finds clocks used in guards in this transition.
                let conjunctions = transition.guard_zone.minimal_constraints().conjunctions;
                for conjunction in &conjunctions {
                    for constraint in conjunction.iter() {
                        edge.guard_dependencies.insert(constraint.i);
                        edge.guard_dependencies.insert(constraint.j);
                    }
                }

                graph.edges.push(edge);

                if !graph
                    .nodes
                    .contains_key(&transition.target_locations.id.get_unique_string())
                {
                    worklist.push_back(transition.target_locations);
                }
            }
        }
    }
}
pub fn find_redundant_clocks(system: &TransitionSystemPtr) -> Vec<ClockReductionInstruction> {
    get_analysis_graph(system).find_clock_redundancies()
}

/// Constructs a [ClockAnalysisGraph] where nodes represents locations and Edges represent transitions
pub fn get_analysis_graph(system: &TransitionSystemPtr) -> ClockAnalysisGraph {
    let mut graph: ClockAnalysisGraph = ClockAnalysisGraph::from_dim(system.get_dim());
    find_edges_and_nodes(system, system.get_initial_location().unwrap(), &mut graph);

    graph
}
