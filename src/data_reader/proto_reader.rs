use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;

use edbm::util::constraints::{Conjunction, Constraint, Disjunction, Inequality, RawInequality};
use edbm::zones::OwnedFederation;

use crate::model_objects::{Component, Decision, Declarations, State};
use crate::protobuf_server::services::{
    clock::Clock as ClockEnum, Clock as ProtoClock, ComponentsInfo, Constraint as ProtoConstraint,
    Decision as ProtoDecision, Disjunction as ProtoDisjunction, LocationTree as ProtoLocationTree,
    State as ProtoState,
};
use crate::system::specifics::SpecificLocation;
use crate::transition_systems::{LocationTree, TransitionSystemPtr};

use super::component_loader::parse_components_if_some;

/// Borrows a [`ComponentsInfo`] and returns the corresponding [`Vec`] of [`Component`]s.
pub fn components_info_to_components(components_info: &ComponentsInfo) -> Vec<Component> {
    components_info
        .components
        .iter()
        .flat_map(parse_components_if_some)
        .flatten()
        .collect()
}

/// Consumes a [`ProtoDecision`] and the borrows the [`TransitionsSystemPtr`] it belongs to and returns the corresponding [`Decision`].
///
/// # Panics
/// If:
/// - `proto_decision.source` is `None`.
/// - `proto_decision.edge` is `None`.
pub fn proto_decision_to_decision(
    proto_decision: ProtoDecision,
    system: &TransitionSystemPtr,
) -> Decision {
    let proto_state: ProtoState = proto_decision
        .source
        .expect("A decision must have a source state.");
    let state = proto_state_to_state(proto_state, system);

    let next_proto_state = proto_decision
        .destination
        .expect("A decision must have a destination state.");
    let next_state = proto_state_to_state(next_proto_state, system);

    let action = proto_decision.action;
    assert!(
        !action.is_empty(),
        "A decision must have an action specified."
    );

    Decision {
        state,
        action,
        transition: None,
        next_state,
    }
}

/// Consumes a [`ProtoState`] and the borrows the [`TransitionsSystemPtr`] it belongs to and returns the corresponding [`State`].
///
/// # Panics
/// If:
/// - `state.federation` is `None`.
/// - `state.location_tree` is `None`.
pub fn proto_state_to_state(state: ProtoState, system: &TransitionSystemPtr) -> State {
    let proto_zone: ProtoDisjunction = state.zone.unwrap();
    let federation: OwnedFederation = proto_zone_to_owned_federation(proto_zone, system);

    let proto_location_tree: ProtoLocationTree = state.location_tree.unwrap();
    let location_tree = proto_location_tree_to_location_tree(proto_location_tree, system);

    // Ensure that the invariants are applied to the state
    let federation = location_tree.apply_invariants(federation);

    State::new(location_tree, federation)
}

fn proto_location_tree_to_location_tree(
    location_tree: ProtoLocationTree,
    system: &TransitionSystemPtr,
) -> Rc<LocationTree> {
    let target: SpecificLocation = location_tree.into();

    system.construct_location_tree(target).unwrap()
}

fn proto_constraint_to_constraint(
    proto_constraint: ProtoConstraint,
    map: &HashMap<u32, (String, &Declarations)>,
) -> Constraint {
    fn determine_index(clock: ProtoClock, map: &HashMap<u32, (String, &Declarations)>) -> usize {
        match clock.clock.unwrap() {
            ClockEnum::ComponentClock(clock) => {
                let comp = clock.component_instance.as_ref().unwrap();
                let (name, decl) = map.get(&comp.component_index).unwrap();
                assert_eq!(name, &comp.component_name);
                *decl.get_clock_index_by_name(&clock.clock_name).unwrap()
            }
            ClockEnum::SystemClock(sc) => sc.clock_index.try_into().unwrap(),
            ClockEnum::ZeroClock(_) => 0,
        }
    }

    let x_clock = proto_constraint.x.unwrap();
    let i = determine_index(x_clock, map);

    let y_clock = proto_constraint.y.unwrap();
    let j = determine_index(y_clock, map);

    let inequality = match proto_constraint.strict {
        true => Inequality::LS(proto_constraint.c),
        false => Inequality::LE(proto_constraint.c),
    };

    Constraint::new(i, j, RawInequality::from_inequality(&inequality))
}

fn proto_zone_to_owned_federation(
    proto_zone: ProtoDisjunction,
    system: &TransitionSystemPtr,
) -> OwnedFederation {
    // Get the vector of conjunctions from the proto
    let proto_conjunctions = proto_zone.conjunctions;

    // Generate map from component index to declarations (include component name for sanity check)
    let infos = system.comp_infos();
    let map = infos
        .iter()
        .map(|c| (c.id, (c.name.clone(), &c.declarations)))
        .collect::<HashMap<_, _>>();

    // Convert the proto conjunctions to real conjunctions
    let conjunctions = proto_conjunctions
        .into_iter()
        .map(|c| {
            Conjunction::new(
                c.constraints
                    .into_iter()
                    .map(|c| proto_constraint_to_constraint(c, &map))
                    .collect(),
            )
        })
        .collect();

    // Create the disjunction
    let disj = Disjunction::new(conjunctions);

    // Create the federation
    OwnedFederation::from_disjunction(&disj, system.get_dim())
}

#[cfg(test)]
mod tests {
    use crate::{system::specifics::SpecificState, tests::refinement::helper::json_get_system};

    use super::*;

    use test_case::test_case;

    const PATH: &str = "samples/json/EcdarUniversity";

    fn assert_state_equals(state1: &State, state2: &State) {
        assert!(
            state1.ref_zone().equals(state2.ref_zone()),
            "Zones are not equal"
        );
        assert_eq!(
            state1.decorated_locations, state2.decorated_locations,
            "Location trees are not equal"
        );
    }

    fn convert_to_proto_and_back(state: &State, system: &TransitionSystemPtr) -> State {
        let specific_state = SpecificState::from_state(state, &**system);
        let proto_state: ProtoState = specific_state.into();
        proto_state_to_state(proto_state, system)
    }

    #[test_case(PATH, "Researcher"; "Researcher state")]
    #[test_case(PATH, "Machine"; "Machine state")]
    #[test_case(PATH, "Machine || Researcher || Administration"; "Comp state")]
    #[test_case(PATH, "Spec"; "Spec state")]
    #[test_case(PATH, "Spec // Machine"; "Machine Spec state")]
    #[test_case(PATH, "Spec // Administration"; "Administration Spec state")]
    #[test_case(PATH, "Spec // Researcher"; "Researcher Spec state")]
    #[test_case(PATH, "Spec // Researcher // Administration"; "Researcher Administration Spec state")]
    #[test_case(PATH, "Spec // Researcher // Machine"; "Researcher Machine Spec state")]
    #[test_case(PATH, "Spec // Machine // Administration"; "Machine Administration Spec state")]
    fn initial_state_conversion_test(path: &str, query: &str) {
        let system = json_get_system(path, query);
        let initial_state = system.get_initial_state().unwrap();
        let initial_state2 = convert_to_proto_and_back(&initial_state, &system);

        assert_state_equals(&initial_state, &initial_state2)
    }

    #[test_case(PATH, "Researcher"; "Researcher state")]
    #[test_case(PATH, "Machine"; "Machine state")]
    #[test_case(PATH, "Machine || Researcher || Administration"; "Comp state")]
    #[test_case(PATH, "Spec"; "Spec state")]
    #[test_case(PATH, "Spec // Machine"; "Machine Spec state")]
    #[test_case(PATH, "Spec // Administration"; "Administration Spec state")]
    #[test_case(PATH, "Spec // Researcher"; "Researcher Spec state")]
    #[test_case(PATH, "Spec // Researcher // Administration"; "Researcher Administration Spec state")]
    #[test_case(PATH, "Spec // Researcher // Machine"; "Researcher Machine Spec state")]
    #[test_case(PATH, "Spec // Machine // Administration"; "Machine Administration Spec state")]
    fn next_state_conversion_test(path: &str, query: &str) {
        let system = json_get_system(path, query);
        let initial_state = system.get_initial_state().unwrap();

        fn rec_test_next(state: &State, system: &TransitionSystemPtr, depth: usize) {
            if depth == 0 {
                return;
            }
            for action in system.get_actions() {
                for t in system.next_transitions(Rc::clone(&state.decorated_locations), &action) {
                    let state = t.use_transition_alt(state);
                    if let Some(state) = state {
                        let next_state = convert_to_proto_and_back(&state, system);
                        assert_state_equals(&state, &next_state);
                        rec_test_next(&state, system, depth - 1);
                    };
                }
            }
        }

        // Explore the 3-step neighbourhood of the initial state and ensure that the conversion is correct
        rec_test_next(&initial_state, &system, 3);
    }

    #[test]
    fn empty_state_test() {
        let system = json_get_system(PATH, "Spec // Machine // Administration");
        let mut initial_state = system.get_initial_state().unwrap();
        initial_state.update_zone(|zone| zone.set_empty());
        let initial_state2 = convert_to_proto_and_back(&initial_state, &system);
        assert_state_equals(&initial_state, &initial_state2)
    }
}
