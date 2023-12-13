use crate::model_objects::expressions::BoolExpression;
use crate::model_objects::{Component, Declarations, Location, LocationType, SyncType};
use crate::transition_systems::{LocationTree, TransitionSystemPtr};
use std::collections::HashMap;
use std::rc::Rc;

pub enum PruningStrategy {
    Reachable,
    NoPruning,
}

use crate::model_objects::Edge;
use edbm::util::constraints::ClockIndex;
use PruningStrategy::*;

pub fn combine_components(
    system: &TransitionSystemPtr,
    reachability: PruningStrategy,
) -> Component {
    let mut location_trees = vec![];
    let mut edges = vec![];
    let clocks = get_clock_map(system);
    match reachability {
        Reachable => {
            collect_reachable_edges_and_locations(system, &mut location_trees, &mut edges, &clocks)
        }
        NoPruning => {
            collect_all_edges_and_locations(system, &mut location_trees, &mut edges, &clocks)
        }
    };

    let locations = get_locations_from_trees(location_trees.as_slice(), &clocks);

    Component {
        name: "".to_string(),
        declarations: Declarations {
            ints: HashMap::new(),
            clocks,
        },
        locations,
        edges,
        special_id: None,
    }
}

pub fn get_locations_from_trees(
    location_trees: &[Rc<LocationTree>],
    clock_map: &HashMap<String, ClockIndex>,
) -> Vec<Location> {
    location_trees
        .iter()
        .cloned()
        .map(|loc_vec| {
            let invariant: Option<BoolExpression> = loc_vec.get_invariants().and_then(|fed| {
                BoolExpression::from_disjunction(&fed.minimal_constraints(), clock_map)
            });

            let location_type = if loc_vec.is_initial() {
                LocationType::Initial
            } else {
                LocationType::Normal
            };

            Location {
                id: loc_vec.id.to_string(),
                invariant,
                location_type,
                urgency: "NORMAL".to_string(), //TODO: Handle different urgencies eventually
            }
        })
        .collect()
}

pub fn get_clock_map(sysrep: &TransitionSystemPtr) -> HashMap<String, ClockIndex> {
    let mut clocks = HashMap::new();
    let decls = sysrep.get_decls();

    if decls.len() == 1 {
        return decls[0].clocks.clone();
    }
    for (comp_id, decl) in decls.into_iter().enumerate() {
        for (k, v) in &decl.clocks {
            if clocks.contains_key(k) {
                clocks.insert(format!("{}{}", k, comp_id), *v);
            } else {
                clocks.insert(k.clone(), *v);
            }
        }
    }

    clocks
}

fn collect_all_edges_and_locations(
    representation: &TransitionSystemPtr,
    locations: &mut Vec<Rc<LocationTree>>,
    edges: &mut Vec<Edge>,
    clock_map: &HashMap<String, ClockIndex>,
) {
    let l = representation.get_all_locations();
    locations.extend(l);
    for location in locations {
        collect_edges_from_location(Rc::clone(location), representation, edges, clock_map);
    }
}

fn collect_reachable_edges_and_locations(
    representation: &TransitionSystemPtr,
    locations: &mut Vec<Rc<LocationTree>>,
    edges: &mut Vec<Edge>,
    clock_map: &HashMap<String, ClockIndex>,
) {
    let l = representation.get_initial_location();

    if l.is_none() {
        return;
    }
    let l = l.unwrap();

    locations.push(l.clone());

    collect_reachable_locations(l, representation, locations);

    for loc in locations {
        collect_edges_from_location(Rc::clone(loc), representation, edges, clock_map);
    }
}

fn collect_reachable_locations(
    location: Rc<LocationTree>,
    representation: &TransitionSystemPtr,
    locations: &mut Vec<Rc<LocationTree>>,
) {
    for input in [true, false].iter() {
        for sync in if *input {
            representation.get_input_actions()
        } else {
            representation.get_output_actions()
        } {
            let transitions = representation.next_transitions(Rc::clone(&location), &sync);

            for transition in transitions {
                let target_location = transition.target_locations;

                if !locations.contains(&target_location) {
                    locations.push(Rc::clone(&target_location));
                    collect_reachable_locations(target_location, representation, locations);
                }
            }
        }
    }
}

fn collect_edges_from_location(
    location: Rc<LocationTree>,
    representation: &TransitionSystemPtr,
    edges: &mut Vec<Edge>,
    clock_map: &HashMap<String, ClockIndex>,
) {
    collect_specific_edges_from_location(
        Rc::clone(&location),
        representation,
        edges,
        true,
        clock_map,
    );
    collect_specific_edges_from_location(
        Rc::clone(&location),
        representation,
        edges,
        false,
        clock_map,
    );
}

fn collect_specific_edges_from_location(
    location: Rc<LocationTree>,
    representation: &TransitionSystemPtr,
    edges: &mut Vec<Edge>,
    input: bool,
    clock_map: &HashMap<String, ClockIndex>,
) {
    for sync in if input {
        representation.get_input_actions()
    } else {
        representation.get_output_actions()
    } {
        let transitions = representation.next_transitions(Rc::clone(&location), &sync);
        for transition in transitions {
            let target_location_id = transition.target_locations.id.to_string();

            let guard = transition.get_renamed_guard_expression(clock_map);
            if let Some(BoolExpression::Bool(false)) = guard {
                continue;
            }

            let edge = Edge {
                id: transition.id.to_string(),
                source_location: location.id.to_string(),
                target_location: target_location_id,
                sync_type: if input {
                    SyncType::Input
                } else {
                    SyncType::Output
                },
                guard,
                update: transition.get_renamed_updates(clock_map),
                sync: sync.clone(),
            };
            edges.push(edge);
        }
    }
}
