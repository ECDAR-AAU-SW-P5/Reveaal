use std::collections::HashSet;

use dyn_clone::{clone_trait_object, DynClone};
use edbm::{
    util::{bounds::Bounds, constraints::ClockIndex},
    zones::OwnedFederation,
};
use log::warn;

use crate::{
    ModelObjects::component::{Declarations, State, Transition},
    System::{query_failures::DeterminismResult, specifics::SpecificLocation},
};
use crate::{System::query_failures::ConsistencyResult, TransitionSystems::CompositionType};

use super::{LocationTuple, TransitionSystem, TransitionSystemPtr};

pub(super) trait ComposedTransitionSystem: DynClone {
    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition>;

    fn check_local_consistency(&self) -> ConsistencyResult;

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr);

    fn get_children_mut(&mut self) -> (&mut TransitionSystemPtr, &mut TransitionSystemPtr);

    fn get_composition_type(&self) -> CompositionType;

    fn get_dim(&self) -> ClockIndex;

    fn get_input_actions(&self) -> HashSet<String>;

    fn get_output_actions(&self) -> HashSet<String>;
}

clone_trait_object!(ComposedTransitionSystem);

impl<T: ComposedTransitionSystem> TransitionSystem for T {
    fn get_local_max_bounds(&self, loc: &LocationTuple) -> Bounds {
        if loc.is_universal() || loc.is_inconsistent() {
            // TODO: this seems wrong
            Bounds::new(self.get_dim())
        } else {
            let (left, right) = self.get_children();
            let loc_l = loc.get_left();
            let loc_r = loc.get_right();
            let mut bounds_l = left.get_local_max_bounds(loc_l);
            let bounds_r = right.get_local_max_bounds(loc_r);
            bounds_l.add_bounds(&bounds_r);
            bounds_l
        }
    }

    fn get_dim(&self) -> ClockIndex {
        self.get_dim()
    }
    fn next_transitions(&self, location: &LocationTuple, action: &str) -> Vec<Transition> {
        self.next_transitions(location, action)
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

    fn get_all_locations(&self) -> Vec<LocationTuple> {
        let (left, right) = self.get_children();
        let mut location_tuples = vec![];
        let left = left.get_all_locations();
        let right = right.get_all_locations();
        for loc1 in &left {
            for loc2 in &right {
                location_tuples.push(LocationTuple::compose(
                    loc1,
                    loc2,
                    self.get_composition_type(),
                ));
            }
        }
        location_tuples
    }

    /// Returns the declarations of both children.
    fn get_decls(&self) -> Vec<&Declarations> {
        let (left, right) = self.get_children();

        let mut comps = left.get_decls();
        comps.extend(right.get_decls());
        comps
    }

    fn check_determinism(&self) -> DeterminismResult {
        let (left, right) = self.get_children();
        left.check_determinism()?;
        right.check_determinism()
    }

    fn check_local_consistency(&self) -> ConsistencyResult {
        let (left, right) = self.get_children();
        left.check_local_consistency()?;
        right.check_local_consistency()
    }

    fn get_initial_state(&self) -> Option<State> {
        let init_loc = self.get_initial_location().unwrap();
        let mut zone = OwnedFederation::init(self.get_dim());
        zone = init_loc.apply_invariants(zone);
        if zone.is_empty() {
            warn!("Empty initial state");
            return None;
        }

        Some(State::create(init_loc, zone))
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        self.get_children()
    }

    fn get_composition_type(&self) -> CompositionType {
        self.get_composition_type()
    }

    fn construct_location_tuple(&self, target: SpecificLocation) -> Result<LocationTuple, String> {
        let (left, right) = self.get_children();
        let (t_left, t_right) = target.split();
        let loc_l = left.construct_location_tuple(t_left)?;
        let loc_r = right.construct_location_tuple(t_right)?;
        Ok(LocationTuple::compose(
            &loc_l,
            &loc_r,
            self.get_composition_type(),
        ))
    }
}

pub trait CollectionOperation<T: Eq + std::hash::Hash> {
    fn is_disjoint_action(&self, other: &HashSet<T>) -> Result<bool, Vec<String>>;
}

impl CollectionOperation<String> for HashSet<String> {
    fn is_disjoint_action(&self, other: &HashSet<String>) -> Result<bool, Vec<String>> {
        let out = self.intersection(other).cloned().collect::<Vec<String>>();
        if !out.is_empty() {
            Err(out)
        } else {
            Ok(true)
        }
    }
}
