use std::{collections::HashSet, rc::Rc};

use dyn_clone::{clone_trait_object, DynClone};
use edbm::{
    util::{bounds::Bounds, constraints::ClockIndex},
    zones::OwnedFederation,
};
use log::warn;

use crate::model_objects::{Declarations, State, Transition};
use crate::system::{
    query_failures::{ConsistencyResult, DeterminismResult},
    specifics::SpecificLocation,
};
use crate::transition_systems::CompositionType;

use super::{LocationTree, TransitionSystem, TransitionSystemPtr};

pub(super) trait ComposedTransitionSystem: DynClone {
    fn next_transitions(&self, location: Rc<LocationTree>, action: &str) -> Vec<Transition>;

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
    fn get_local_max_bounds(&self, loc: &LocationTree) -> Bounds {
        let (left, right) = self.get_children();
        let loc_l = loc.get_left();
        let loc_r = loc.get_right();
        let mut bounds_l = left.get_local_max_bounds(&loc_l);
        let bounds_r = right.get_local_max_bounds(&loc_r);
        bounds_l.add_bounds(&bounds_r);
        bounds_l
    }

    fn get_dim(&self) -> ClockIndex {
        self.get_dim()
    }
    fn next_transitions(&self, location: Rc<LocationTree>, action: &str) -> Vec<Transition> {
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

    fn get_initial_location(&self) -> Option<Rc<LocationTree>> {
        let (left, right) = self.get_children();
        let l = left.get_initial_location()?;
        let r = right.get_initial_location()?;

        Some(LocationTree::compose(l, r, self.get_composition_type()))
    }

    fn get_all_locations(&self) -> Vec<Rc<LocationTree>> {
        let (left, right) = self.get_children();
        let mut location_trees: Vec<Rc<_>> = vec![];
        let left = left.get_all_locations();
        let right = right.get_all_locations();
        for loc1 in &left {
            for loc2 in &right {
                location_trees.push(LocationTree::compose(
                    Rc::clone(loc1),
                    Rc::clone(loc2),
                    self.get_composition_type(),
                ));
            }
        }
        location_trees
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
        let init_loc = self.get_initial_location()?;
        let mut zone = OwnedFederation::init(self.get_dim());
        zone = init_loc.apply_invariants(zone);
        if zone.is_empty() {
            warn!("Empty initial state");
            return None;
        }

        Some(State::new(init_loc, zone))
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        self.get_children()
    }

    fn get_composition_type(&self) -> CompositionType {
        self.get_composition_type()
    }

    fn construct_location_tree(
        &self,
        target: SpecificLocation,
    ) -> Result<Rc<LocationTree>, String> {
        let (left, right) = self.get_children();
        let (t_left, t_right) = target.split();
        let loc_l = left.construct_location_tree(t_left)?;
        let loc_r = right.construct_location_tree(t_right)?;
        Ok(LocationTree::compose(
            loc_l,
            loc_r,
            self.get_composition_type(),
        ))
    }
}
