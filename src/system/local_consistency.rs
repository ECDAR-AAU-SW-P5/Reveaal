use std::rc::Rc;

use edbm::zones::OwnedFederation;
use log::warn;

use crate::model_objects::State;
use crate::system::query_failures::{ConsistencyFailure, DeterminismFailure};
use crate::transition_systems::TransitionSystem;

use super::query_failures::{ConsistencyResult, DeterminismResult};

///Local consistency check WITH pruning.
pub fn is_least_consistent(system: &dyn TransitionSystem) -> ConsistencyResult {
    if let Some(mut state) = system.get_initial_state() {
        let mut passed = vec![];
        state.extrapolate_max_bounds(system);
        consistency_least_helper(state, &mut passed, system)
    } else {
        warn!("Empty initial state");
        ConsistencyFailure::no_initial_state(system)
    }
}

///Checks if a [TransitionSystem] is deterministic.
pub fn check_determinism(system: &dyn TransitionSystem) -> DeterminismResult {
    let mut passed = vec![];
    let state = system.get_initial_state();
    if state.is_none() {
        return Ok(());
    }
    let mut state = state.unwrap();
    state.update_zone(|_| OwnedFederation::universe(system.get_dim()));
    is_deterministic_helper(state, &mut passed, system)
}

fn is_deterministic_helper(
    state: State,
    passed_list: &mut Vec<State>,
    system: &dyn TransitionSystem,
) -> DeterminismResult {
    if state.is_contained_in_list(passed_list) {
        return Ok(());
    }

    passed_list.push(state.clone());

    for action in system.get_actions() {
        let mut location_fed = OwnedFederation::empty(system.get_dim());
        for transition in &system.next_transitions(Rc::clone(&state.decorated_locations), &action) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                let mut allowed_fed = transition.get_allowed_federation();
                allowed_fed = state.decorated_locations.apply_invariants(allowed_fed);
                if allowed_fed.has_intersection(&location_fed) {
                    warn!(
                        "Not deterministic from location {} failing action {}",
                        state.decorated_locations.id, action
                    );
                    return DeterminismFailure::from_system_and_action(system, action, &state);
                }
                location_fed += allowed_fed;
                new_state.extrapolate_max_bounds(system);

                is_deterministic_helper(new_state, passed_list, system)?;
            }
        }
    }
    Ok(())
}

/// Local consistency check WITHOUT pruning
pub fn is_fully_consistent(system: &dyn TransitionSystem) -> ConsistencyResult {
    let mut passed = vec![];
    let state = system.get_initial_state();
    if state.is_none() {
        warn!("Empty initial state");
        return ConsistencyFailure::no_initial_state(system);
    }
    consistency_fully_helper(state.unwrap(), &mut passed, system)
}

pub fn consistency_least_helper(
    state: State,
    passed_list: &mut Vec<State>,
    system: &dyn TransitionSystem,
) -> ConsistencyResult {
    if state.is_contained_in_list(passed_list) {
        return Ok(());
    }
    if state.decorated_locations.is_universal() {
        return Ok(());
    }
    if state.decorated_locations.is_inconsistent() {
        return ConsistencyFailure::inconsistent(system, &state);
    }

    passed_list.push(state.clone());

    for input in system.get_input_actions() {
        for transition in &system.next_inputs(Rc::clone(&state.decorated_locations), &input) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.extrapolate_max_bounds(system);

                consistency_least_helper(new_state, passed_list, system)?;
            }
        }
    }

    if state.ref_zone().can_delay_indefinitely() {
        return Ok(());
    }

    for output in system.get_output_actions() {
        for transition in system.next_outputs(Rc::clone(&state.decorated_locations), &output) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.extrapolate_max_bounds(system);
                if let Ok(()) = consistency_least_helper(new_state, passed_list, system) {
                    return Ok(());
                }
            }
        }
    }
    warn!("No saving outputs from {}", state.decorated_locations.id);
    ConsistencyFailure::inconsistent_from(system, &state)
}

fn consistency_fully_helper(
    state: State,
    passed_list: &mut Vec<State>,
    system: &dyn TransitionSystem,
) -> ConsistencyResult {
    if state.is_contained_in_list(passed_list) {
        return Ok(());
    }
    passed_list.push(state.clone());

    for input in system.get_input_actions() {
        for transition in system.next_inputs(Rc::clone(&state.decorated_locations), &input) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.extrapolate_max_bounds(system);
                if new_state.is_subset_of(&state) {
                    continue;
                }
                consistency_fully_helper(new_state, passed_list, system)?;
            }
        }
    }

    let mut output_existed = false;
    for output in system.get_output_actions() {
        for transition in system.next_outputs(Rc::clone(&state.decorated_locations), &output) {
            let mut new_state = state.clone();
            if transition.use_transition(&mut new_state) {
                new_state.extrapolate_max_bounds(system);
                if new_state.is_subset_of(&state) {
                    continue;
                }

                output_existed = true;

                consistency_fully_helper(new_state, passed_list, system)?;
            }
        }
    }
    if output_existed {
        Ok(())
    } else {
        let last_state = passed_list.last().unwrap();
        match last_state.ref_zone().can_delay_indefinitely() {
            false => ConsistencyFailure::inconsistent_from(system, &state),
            true => Ok(()),
        }
    }
}
