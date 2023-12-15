use edbm::util::bounds::Bounds;
use edbm::zones::OwnedFederation;

use super::query_failures::PathFailure;
use super::specifics::SpecificPath;
use crate::model_objects::{Decision, State, Transition};
use crate::transition_systems::{LocationID, TransitionSystemPtr};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use super::query_failures::PathResult;

/// This holds the result of a reachability query
#[derive(Debug, Clone)]
pub struct Path {
    pub path: Vec<Decision>,
}

/// This holds which transition from which state (the `destination_state` of the `previous_sub_path`) it took to reach this state
struct SubPath {
    previous_sub_path: Option<Rc<SubPath>>,
    destination_state: State,
    transition: Option<(Transition, String)>,
}

fn is_trivially_unreachable(start_state: &State, end_state: &State) -> bool {
    // If any of the zones are empty
    if start_state.ref_zone().is_empty() || end_state.ref_zone().is_empty() {
        return true;
    }

    // If the end location has invariants and these do not have an intersection (overlap) with the zone of the end state of the query
    if let Some(invariants) = end_state.decorated_locations.get_invariants() {
        if !end_state.ref_zone().has_intersection(invariants) {
            return true;
        }
    }

    false
}

///# Find path
///
/// Returns a path from a start state to an end state in a transition system.
///
/// If it is reachable, it returns a path.
///
/// If it is not reachable, it returns None.
///
/// The start state can be omitted with None to use the start state of the transition system.
///
///## Checking if a state can reach another:
/// ```ignore
/// let is_reachable: bool = match find_path(Some(start_state), end_state, transition_system) {
///    Ok(result) => match result {
///        Some(path) => true,
///        None => false,
///    },
///    Err(string) => panic!(string),
/// };
/// ```
///
///## Omitting start state:
/// ```ignore
/// let is_reachable: bool = match find_path(None, end_state, transition_system) {
///    Ok(result) => match result {
///        Some(path) => true,
///        None => false,
///    },
///    Err(string) => panic!(string),
/// };
/// ```
pub fn find_path(
    start_state: State,
    end_state: State,
    system: &TransitionSystemPtr,
) -> Result<Path, PathFailure> {
    if is_trivially_unreachable(&start_state, &end_state) {
        return Err(PathFailure::Unreachable);
    }

    reachability_search(&start_state, &end_state, system)
}

pub fn find_specific_path(
    start_state: State,
    end_state: State,
    system: &TransitionSystemPtr,
) -> PathResult {
    find_path(start_state, end_state, system).map(|p| SpecificPath::from_path(&p, system.as_ref()))
}

/// Currently runs a BFS search on the transition system.
/// BFS is preferable to a DFS, as it reduces the chance of "Mistakes", meaning
/// having to revisit a state with a larger zone, forcing it to be readded ot the frontier.
/// Inspired from http://link.springer.com/10.1007/978-3-319-22975-1_9, see article for possible optimizations and more explanation.
fn reachability_search(
    start_state: &State,
    end_state: &State,
    system: &TransitionSystemPtr,
) -> Result<Path, PathFailure> {
    // Apply the invariant of the start state to the start state
    let mut start_state = start_state.clone();
    start_state.apply_invariants();

    // hashmap linking every location to all its current zones
    let mut visited_states: HashMap<LocationID, Vec<OwnedFederation>> = HashMap::new();

    // List of states that are to be visited
    let mut frontier_states: VecDeque<Rc<SubPath>> = VecDeque::new();

    let mut actions: Vec<String> = system.get_actions().into_iter().collect();
    actions.sort();

    // Push start state to visited state
    visited_states.insert(
        start_state.decorated_locations.id.clone(),
        vec![start_state.ref_zone().clone()],
    );

    // Push initial state to frontier
    frontier_states.push_back(Rc::new(SubPath {
        previous_sub_path: None,
        destination_state: start_state.clone(),
        transition: None,
    }));

    let target_bounds = end_state.ref_zone().get_bounds();

    // Take the first state from the frontier and explore it
    while let Some(sub_path) = frontier_states.pop_front() {
        if reached_end_state(&sub_path.destination_state, end_state) {
            return Ok(make_path(sub_path, start_state));
        }

        for action in &actions {
            for transition in &system.next_transitions(
                Rc::clone(&sub_path.destination_state.decorated_locations),
                action,
            ) {
                take_transition(
                    &sub_path,
                    transition,
                    &mut frontier_states,
                    &mut visited_states,
                    system,
                    action,
                    &target_bounds,
                );
            }
        }
    }
    // If nothing has been found, it is not reachable
    Err(PathFailure::Unreachable)
}

fn reached_end_state(cur_state: &State, end_state: &State) -> bool {
    cur_state
        .decorated_locations
        .compare_partial_locations(Rc::clone(&end_state.decorated_locations))
        && cur_state.ref_zone().has_intersection(end_state.ref_zone())
}

fn take_transition(
    sub_path: &Rc<SubPath>,
    transition: &Transition,
    frontier_states: &mut VecDeque<Rc<SubPath>>,
    visited_states: &mut HashMap<LocationID, Vec<OwnedFederation>>,
    system: &TransitionSystemPtr,
    action: &str,
    target_bounds: &Bounds,
) {
    let mut new_state = sub_path.destination_state.clone();
    if transition.use_transition(&mut new_state) {
        // Extrapolation ensures the bounds cant grow indefinitely, avoiding infinite loops
        // We must take the added bounds from the target state into account to ensure correctness
        new_state.extrapolate_max_bounds_with_extra_bounds(system.as_ref(), target_bounds);
        let new_location_id = &new_state.decorated_locations.id;
        let existing_zones = visited_states.entry(new_location_id.clone()).or_default();
        // If this location has not already been reached (explored) with a larger zone
        if !zone_subset_of_existing_zones(new_state.ref_zone(), existing_zones) {
            // Remove the smaller zones for this location in visited_states
            remove_existing_subsets_of_zone(new_state.ref_zone(), existing_zones);
            // Add the new zone to the list of zones for this location in visited_states
            visited_states
                .get_mut(new_location_id)
                .unwrap()
                .push(new_state.ref_zone().clone());
            // Add the new state to the frontier
            frontier_states.push_back(Rc::new(SubPath {
                previous_sub_path: Some(Rc::clone(sub_path)),
                destination_state: new_state,
                transition: Some((transition.clone(), action.to_string())),
            }));
        }
    }
}

/// Checks if this zone is redundant by being a subset of any other zone
fn zone_subset_of_existing_zones(
    new_state: &OwnedFederation,
    existing_states: &[OwnedFederation],
) -> bool {
    existing_states
        .iter()
        .any(|existing_state| new_state.subset_eq(existing_state))
}

/// Removes everything in existing_zones that is a subset of zone
fn remove_existing_subsets_of_zone(
    new_zone: &OwnedFederation,
    existing_zones: &mut Vec<OwnedFederation>,
) {
    existing_zones.retain(|existing_zone| !existing_zone.subset_eq(new_zone));
}
/// Makes the path from the last subpath
fn make_path(mut sub_path: Rc<SubPath>, start_state: State) -> Path {
    let mut path: Vec<(Transition, String)> = Vec::new();
    // Traverse the subpaths to make the path (from end location to start location)
    while sub_path.previous_sub_path.is_some() {
        path.push(sub_path.transition.clone().unwrap());
        sub_path = Rc::clone(sub_path.previous_sub_path.as_ref().unwrap());
    }
    // Reverse the path since the transitions are in reverse order (now from start location to end location)
    path.reverse();
    let mut state = start_state;

    let mut decisions = Vec::new();

    for (transition, action) in path {
        let decision = Decision::from_state_transition(state.clone(), &transition, action)
            .expect("If the transition is in a path, it should lead to a non-empty state");

        decisions.push(decision);

        transition.use_transition(&mut state);
    }

    Path { path: decisions }
}
