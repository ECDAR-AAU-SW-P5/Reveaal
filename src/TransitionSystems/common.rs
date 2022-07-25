use std::collections::HashSet;

use dyn_clone::{clone_trait_object, DynClone};

use crate::{
    DBMLib::dbm::Federation,
    ModelObjects::{
        component::{Declarations, State, Transition},
        max_bounds::MaxBounds,
    },
};

use super::{CompositionType, LocationTuple, TransitionSystem, TransitionSystemPtr};

pub trait ComposedTransitionSystem: DynClone {
    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition>;

    fn is_locally_consistent(&self) -> bool;

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr);

    fn get_composition_type(&self) -> CompositionType;

    fn get_dim(&self) -> u32;

    fn get_input_actions(&self) -> HashSet<String>;

    fn get_output_actions(&self) -> HashSet<String>;
}

clone_trait_object!(ComposedTransitionSystem);

impl<T: ComposedTransitionSystem> TransitionSystem for T {
    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        self.next_transitions(location, action)
    }
    fn get_max_bounds(&self) -> MaxBounds {
        let (left, right) = self.get_children();

        let mut bounds = left.get_max_bounds();
        bounds.add_bounds(&right.get_max_bounds());
        bounds
    }
    fn get_input_actions(&self) -> HashSet<String> {
        self.get_input_actions()
    }
    fn get_output_actions(&self) -> HashSet<String> {
        self.get_output_actions()
    }
    fn get_actions(&self) -> HashSet<String> {
        self.get_input_actions()
            .union(&self.get_output_actions())
            .map(|action| action.to_string())
            .collect()
    }
    fn get_initial_location(&self) -> Option<LocationTuple> {
        let (left, right) = self.get_children();
        let l = left.get_initial_location()?;
        let r = right.get_initial_location()?;

        Some(LocationTuple::compose(&l, &r, self.get_composition_type()))
    }

    fn get_decls(&self) -> Vec<&Declarations> {
        let (left, right) = self.get_children();

        let mut comps = left.get_decls();
        comps.extend(right.get_decls());
        comps
    }

    fn precheck_sys_rep(&self) -> bool {
        if !self.is_deterministic() {
            println!("NOT DETERMINISTIC");
            return false;
        }

        if !self.is_locally_consistent() {
            println!("NOT CONSISTENT");
            return false;
        }
        true
    }

    fn is_deterministic(&self) -> bool {
        let (left, right) = self.get_children();
        left.is_deterministic() && right.is_deterministic()
    }

    fn get_initial_state(&self) -> Option<State> {
        let init_loc = self.get_initial_location().unwrap();
        let mut zone = Federation::init(self.get_dim());
        if !init_loc.apply_invariants(&mut zone) {
            println!("Empty initial state");
            return None;
        }

        Some(State {
            decorated_locations: init_loc,
            zone,
        })
    }

    fn get_dim(&self) -> u32 {
        self.get_dim()
    }

    fn get_all_locations(&self) -> Vec<LocationTuple> {
        let (left, right) = self.get_children();
        let mut location_tuples = vec![];
        let left = left.get_all_locations();
        let right = right.get_all_locations();
        for loc1 in &left {
            for loc2 in &right {
                location_tuples.push(LocationTuple::compose(
                    &loc1,
                    &loc2,
                    self.get_composition_type(),
                ));
            }
        }
        location_tuples
    }

    fn is_locally_consistent(&self) -> bool {
        self.is_locally_consistent()
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        self.get_children()
    }

    fn get_composition_type(&self) -> CompositionType {
        self.get_composition_type()
    }
}
