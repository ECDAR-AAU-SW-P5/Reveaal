use edbm::util::constraints::ClockIndex;

use crate::model_objects::Transition;
use crate::system::local_consistency;
use crate::system::query_failures::{ActionFailure, ConsistencyResult, SystemRecipeFailure};
use crate::transition_systems::{
    CompositionType, LocationTree, TransitionSystem, TransitionSystemPtr,
};
use std::collections::hash_set::HashSet;
use std::rc::Rc;

use super::common::ComposedTransitionSystem;

#[derive(Clone)]
pub struct Conjunction {
    left: TransitionSystemPtr,
    right: TransitionSystemPtr,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
    dim: ClockIndex,
}

impl Conjunction {
    /// Creates a new [TransitionSystem] that is the conjunction of `left` and `right`.
    pub fn new_ts(
        left: TransitionSystemPtr,
        right: TransitionSystemPtr,
        dim: ClockIndex,
    ) -> Result<TransitionSystemPtr, Box<SystemRecipeFailure>> {
        let left_in = left.get_input_actions();
        let left_out = left.get_output_actions();

        let right_in = right.get_input_actions();
        let right_out = right.get_output_actions();

        if !left_in.is_disjoint(&right_out) {
            return ActionFailure::not_disjoint(
                (left.as_ref(), left_in),
                (right.as_ref(), right_out),
            )
            .map_err(|e| e.to_rfconj(left, right));
        }
        if !left_out.is_disjoint(&right_in) {
            return ActionFailure::not_disjoint(
                (left.as_ref(), left_out),
                (right.as_ref(), right_in),
            )
            .map_err(|e| e.to_rfconj(left, right));
        }

        let outputs = left
            .get_output_actions()
            .intersection(&right.get_output_actions())
            .cloned()
            .collect();

        let inputs = left
            .get_input_actions()
            .intersection(&right.get_input_actions())
            .cloned()
            .collect();

        let ts = Box::new(Conjunction {
            left,
            right,
            inputs,
            outputs,
            dim,
        });
        local_consistency::is_least_consistent(ts.as_ref())
            .map_err(|e| e.to_recipe_failure(ts.as_ref()))?;
        Ok(ts)
    }
}

impl ComposedTransitionSystem for Conjunction {
    fn next_transitions(&self, location: Rc<LocationTree>, action: &str) -> Vec<Transition> {
        assert!(self.actions_contain(action));

        let loc_left = location.get_left();
        let loc_right = location.get_right();

        let left = self.left.next_transitions(loc_left, action);
        let right = self.right.next_transitions(loc_right, action);

        Transition::combinations(&left, &right, CompositionType::Conjunction)
    }

    fn check_local_consistency(&self) -> ConsistencyResult {
        Ok(()) // By definition from the Conjunction::new()
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        (&self.left, &self.right)
    }

    fn get_children_mut(&mut self) -> (&mut TransitionSystemPtr, &mut TransitionSystemPtr) {
        (&mut self.left, &mut self.right)
    }

    fn get_composition_type(&self) -> CompositionType {
        CompositionType::Conjunction
    }

    fn get_dim(&self) -> ClockIndex {
        self.dim
    }

    fn get_input_actions(&self) -> HashSet<String> {
        self.inputs.clone()
    }

    fn get_output_actions(&self) -> HashSet<String> {
        self.outputs.clone()
    }
}
