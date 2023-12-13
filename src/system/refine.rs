use edbm::zones::OwnedFederation;
use log::{debug, info, log_enabled, trace, Level};

use crate::model_objects::{
    PassedStateList, PassedStateListExt, StatePair, Transition, WaitingStateList,
};
use crate::system::query_failures::RefinementFailure;
use crate::transition_systems::TransitionSystemPtr;
use std::collections::HashSet;
use std::rc::Rc;

use super::query_failures::{ActionFailure, RefinementPrecondition, RefinementResult};

const SUCCESS: RefinementResult = Ok(());

enum StatePairResult {
    Valid,
    EmptyTransition2s,
    NotEmptyResult,
    CutsDelaySolutions,
}

impl StatePairResult {
    fn check(
        &self,
        sys1: &TransitionSystemPtr,
        sys2: &TransitionSystemPtr,
        action: &str,
        curr_pair: &StatePair,
    ) -> RefinementResult {
        match self {
            StatePairResult::Valid => Ok(()),
            StatePairResult::EmptyTransition2s | StatePairResult::NotEmptyResult => {
                RefinementFailure::cannot_match(sys1.as_ref(), sys2.as_ref(), action, curr_pair)
            }
            StatePairResult::CutsDelaySolutions => {
                RefinementFailure::cuts_delays(sys1.as_ref(), sys2.as_ref(), action, curr_pair)
            }
        }
    }
}

fn common_actions(
    sys1: &TransitionSystemPtr,
    sys2: &TransitionSystemPtr,
    is_input: bool,
) -> HashSet<String> {
    if is_input {
        sys2.get_input_actions()
    } else {
        sys1.get_output_actions()
    }
}

fn extra_actions(
    sys1: &TransitionSystemPtr,
    sys2: &TransitionSystemPtr,
    is_input: bool,
) -> HashSet<String> {
    if is_input {
        sys2.get_input_actions()
            .difference(&sys1.get_input_actions())
            .cloned()
            .collect()
    } else {
        sys1.get_output_actions()
            .difference(&sys2.get_output_actions())
            .cloned()
            .collect()
    }
}

struct RefinementContext<'a> {
    pub passed_list: PassedStateList,
    pub waiting_list: WaitingStateList,
    pub sys1: &'a TransitionSystemPtr,
    pub sys2: &'a TransitionSystemPtr,
}

impl<'a> RefinementContext<'a> {
    fn new(sys1: &'a TransitionSystemPtr, sys2: &'a TransitionSystemPtr) -> RefinementContext<'a> {
        RefinementContext {
            passed_list: PassedStateList::new(),
            waiting_list: WaitingStateList::new(),
            sys1,
            sys2,
        }
    }
}

/// Checks if sys1 refines sys2
pub fn check_refinement(sys1: TransitionSystemPtr, sys2: TransitionSystemPtr) -> RefinementResult {
    let mut context = RefinementContext::new(&sys1, &sys2);
    let dimensions = sys1.get_dim();
    debug!("Dimensions: {}", dimensions);

    //Firstly we check the preconditions
    check_preconditions(&sys1, &sys2)?;

    // Common inputs and outputs
    let inputs = common_actions(&sys1, &sys2, true);
    let outputs = common_actions(&sys1, &sys2, false);

    info!(
        "Left inputs: {:?}, Left outputs: {:?}",
        sys1.get_input_actions(),
        sys1.get_output_actions()
    );

    info!(
        "Right inputs: {:?}, Right outputs; {:?}",
        sys2.get_input_actions(),
        sys2.get_output_actions()
    );

    // Extra inputs and outputs are ignored by default
    let extra_inputs = extra_actions(&sys1, &sys2, true);
    let extra_outputs = extra_actions(&sys1, &sys2, false);

    let initial_locations_1 = sys1.get_initial_location();
    let initial_locations_2 = sys2.get_initial_location();

    debug!("Extra inputs {:?}", extra_inputs);
    debug!("Extra outputs {:?}", extra_outputs);

    if initial_locations_1.is_none() {
        if initial_locations_2.is_none() {
            // Both are empty, so trivially true
            return SUCCESS;
        }
        return RefinementFailure::empty_child(sys1.as_ref(), sys2.as_ref(), true);
    }

    if initial_locations_2.is_none() {
        //The empty automata cannot implement
        return RefinementFailure::empty_child(sys1.as_ref(), sys2.as_ref(), false);
    }

    let initial_locations_1 = initial_locations_1.unwrap();
    let initial_locations_2 = initial_locations_2.unwrap();

    let mut initial_pair = StatePair::from_locations(
        dimensions,
        Rc::clone(&initial_locations_1),
        Rc::clone(&initial_locations_2),
    );

    if initial_pair.ref_zone().is_empty() {
        return RefinementFailure::empty_initial(sys1.as_ref(), sys2.as_ref());
    }
    initial_pair.extrapolate_max_bounds(context.sys1, context.sys2);

    debug!("Initial {}", initial_pair);
    context.waiting_list.put(initial_pair);

    while !context.waiting_list.is_empty() {
        let curr_pair = context.waiting_list.pop().unwrap();
        trace!("Checking {}", curr_pair);

        context.passed_list.put(curr_pair.clone());
        for output in &outputs {
            let extra = extra_outputs.contains(output);

            let output_transition1 = sys1.next_outputs(curr_pair.get_locations1(), output);
            let output_transition2 = if extra {
                vec![Transition::without_id(
                    curr_pair.get_locations2(),
                    dimensions,
                )]
            } else {
                sys2.next_outputs(curr_pair.get_locations2(), output)
            };

            has_valid_state_pairs(
                &output_transition1,
                &output_transition2,
                &curr_pair,
                &mut context,
                true,
            )
            .check(&sys1, &sys2, output, &curr_pair)?;
        }

        for input in &inputs {
            let extra = extra_inputs.contains(input);

            let input_transitions1 = if extra {
                vec![Transition::without_id(
                    curr_pair.get_locations1(),
                    dimensions,
                )]
            } else {
                sys1.next_inputs(curr_pair.get_locations1(), input)
            };

            let input_transitions2 = sys2.next_inputs(curr_pair.get_locations2(), input);

            has_valid_state_pairs(
                &input_transitions2,
                &input_transitions1,
                &curr_pair,
                &mut context,
                false,
            )
            .check(&sys1, &sys2, input, &curr_pair)?;
        }
    }
    info!("Refinement check passed");
    if log_enabled!(Level::Debug) {
        debug!("With relation:");
        print_relation(&context.passed_list);
    }

    SUCCESS
}

fn print_relation(passed_list: &PassedStateList) {
    let verbose = false;

    let mut sorted_keys: Vec<_> = passed_list.keys().collect();
    sorted_keys.sort_by_key(|(a, b)| format!("1:{}, 2:{}", a, b));
    for (id1, id2) in sorted_keys {
        let zones = passed_list.zones(&(id1.clone(), id2.clone()));

        debug!(
            "{}",
            if zones.len() != 1 {
                format!("1:{} 2:{} {} zones", id1, id2, zones.len())
            } else if verbose {
                format!("1:{} 2:{} {} zone", id1, id2, zones[0])
            } else {
                format!("1:{} 2:{}", id1, id2)
            }
        );
    }
}

fn has_valid_state_pairs(
    transitions1: &[Transition],
    transitions2: &[Transition],
    curr_pair: &StatePair,
    context: &mut RefinementContext,
    is_state1: bool,
) -> StatePairResult {
    let (fed1, fed2) = get_guard_fed_for_sides(transitions1, transitions2, curr_pair, is_state1);

    // If there are no valid transition1s, continue
    if fed1.is_empty() {
        return StatePairResult::Valid;
    }

    // If there are (valid) transition1s but no transition2s there are no valid pairs
    if fed2.is_empty() {
        trace!("Empty transition2s");
        return StatePairResult::EmptyTransition2s;
    };

    let result_federation = fed1.subtraction(&fed2);

    // If the entire zone of transition1s cannot be matched by transition2s
    if !result_federation.is_empty() {
        return StatePairResult::NotEmptyResult;
    }

    // Finally try to create the pairs
    let res = try_create_new_state_pairs(transitions1, transitions2, curr_pair, context, is_state1);

    match res {
        BuildResult::Success => StatePairResult::Valid,
        BuildResult::Failure => StatePairResult::CutsDelaySolutions,
    }
}

fn get_guard_fed_for_sides(
    transitions1: &[Transition],
    transitions2: &[Transition],
    curr_pair: &StatePair,
    is_state1: bool,
) -> (OwnedFederation, OwnedFederation) {
    let dim = curr_pair.ref_zone().dim();

    let pair_zone = curr_pair.ref_zone();
    trace!("Zone: {}", pair_zone);
    //create guard zones left
    let mut feds = OwnedFederation::empty(dim);
    trace!("{}", if is_state1 { "Left:" } else { "Right:" });
    for transition in transitions1 {
        trace!("{}", transition);
        feds += transition.get_allowed_federation();
    }
    let fed1 = feds.intersection(pair_zone);
    trace!("{}", fed1);

    trace!("{}", if is_state1 { "Right:" } else { "Left:" });
    //Create guard zones right
    let mut feds = OwnedFederation::empty(dim);
    for transition in transitions2 {
        trace!("{}", transition);
        feds += transition.get_allowed_federation();
    }
    let fed2 = feds.intersection(pair_zone);
    trace!("{}", fed2);

    (fed1, fed2)
}

enum BuildResult {
    Success,
    Failure,
}

/// Returns a failure if the new state pairs cut delay solutions, otherwise returns success
fn try_create_new_state_pairs(
    transitions1: &[Transition],
    transitions2: &[Transition],
    curr_pair: &StatePair,
    context: &mut RefinementContext,
    is_state1: bool,
) -> BuildResult {
    for transition1 in transitions1 {
        for transition2 in transitions2 {
            if let BuildResult::Failure =
                build_state_pair(transition1, transition2, curr_pair, context, is_state1)
            {
                return BuildResult::Failure;
            }
        }
    }

    BuildResult::Success
}

fn build_state_pair(
    transition1: &Transition,
    transition2: &Transition,
    curr_pair: &StatePair,
    context: &mut RefinementContext,
    is_state1: bool,
) -> BuildResult {
    //Creates DBM for that state pair
    let mut new_sp_zone = curr_pair.clone_zone();

    //Apply guards on both sides

    //Applies the left side guards and checks if zone is valid
    new_sp_zone = transition1.apply_guards(new_sp_zone);
    //Applies the right side guards and checks if zone is valid
    new_sp_zone = transition2.apply_guards(new_sp_zone);

    // Continue to the next transition pair if the zone is empty
    if new_sp_zone.is_empty() {
        return BuildResult::Success;
    }

    //Apply updates on both sides
    new_sp_zone = transition1.apply_updates(new_sp_zone);
    new_sp_zone = transition2.apply_updates(new_sp_zone);

    //Perform a delay on the zone after the updates were applied
    new_sp_zone = new_sp_zone.up();

    //Update locations in states
    let (locations1, locations2) = (
        Rc::clone(&transition1.target_locations),
        Rc::clone(&transition2.target_locations),
    );

    // Apply invariants on the left side of relation
    let (left_loc, right_loc) = if is_state1 {
        (locations1, locations2)
    } else {
        (locations2, locations1)
    };

    new_sp_zone = left_loc.apply_invariants(new_sp_zone);

    // Clone the zone before applying right side invariants
    let s_invariant = new_sp_zone.clone();

    // Apply right side invariants on the zone
    new_sp_zone = right_loc.apply_invariants(new_sp_zone);

    // Continue to the next transition pair if the newly built zones are empty
    if new_sp_zone.is_empty() || s_invariant.is_empty() {
        return BuildResult::Success;
    }

    // inv_s = x<10, inv_t = x>2 -> t cuts solutions but not delays, so it is fine and we can call down:
    let t_invariant = new_sp_zone.clone().down();

    // Check if the invariant of T (right) cuts delay solutions from S (left) and if so, report failure
    if !(s_invariant.subset_eq(&t_invariant)) {
        return BuildResult::Failure;
    }

    let mut new_sp = StatePair::new(left_loc, right_loc, Rc::new(new_sp_zone));
    new_sp.extrapolate_max_bounds(context.sys1, context.sys2);

    if !context.passed_list.has(&new_sp) && !context.waiting_list.has(&new_sp) {
        debug!("New state {}", new_sp);

        context.waiting_list.put(new_sp);
    }

    BuildResult::Success
}

fn check_preconditions(
    sys1: &TransitionSystemPtr,
    sys2: &TransitionSystemPtr,
) -> Result<(), Box<RefinementPrecondition>> {
    sys1.precheck_sys_rep()
        .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))?;
    sys2.precheck_sys_rep()
        .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))?;

    let s_outputs = sys1.get_output_actions();
    let t_outputs = sys2.get_output_actions();

    let s_inputs = sys1.get_input_actions();
    let t_inputs = sys2.get_input_actions();

    if !s_inputs.is_disjoint(&t_outputs) {
        ActionFailure::not_disjoint((sys1.as_ref(), s_inputs), (sys2.as_ref(), t_outputs))
            .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))
    } else if !t_inputs.is_disjoint(&s_outputs) {
        ActionFailure::not_disjoint((sys2.as_ref(), t_inputs), (sys1.as_ref(), s_outputs))
            .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))
    } else if !s_inputs.is_subset(&t_inputs) {
        ActionFailure::not_subset((sys1.as_ref(), s_inputs), (sys2.as_ref(), t_inputs))
            .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))
    } else if !t_outputs.is_subset(&s_outputs) {
        ActionFailure::not_subset((sys2.as_ref(), t_outputs), (sys1.as_ref(), s_outputs))
            .map_err(|e| e.to_precondition(sys1.as_ref(), sys2.as_ref()))
    } else {
        Ok(())
    }
}
