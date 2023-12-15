use edbm::util::constraints::ClockIndex;

use crate::model_objects::Transition;
use crate::system::query_failures::{ActionFailure, SystemRecipeFailure};
use crate::transition_systems::{LocationTree, TransitionSystem, TransitionSystemPtr};
use std::collections::hash_set::HashSet;
use std::rc::Rc;

use super::common::ComposedTransitionSystem;
use super::CompositionType;

#[derive(Clone)]
pub struct Composition {
    left: TransitionSystemPtr,
    right: TransitionSystemPtr,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
    left_unique_actions: HashSet<String>,
    right_unique_actions: HashSet<String>,
    common_actions: HashSet<String>,

    dim: ClockIndex,
}

impl Composition {
    /// Creates a new [TransitionSystem] that is the composition of `left` and `right`.
    pub fn new_ts(
        left: TransitionSystemPtr,
        right: TransitionSystemPtr,
        dim: ClockIndex,
    ) -> Result<TransitionSystemPtr, Box<SystemRecipeFailure>> {
        let left_in = left.get_input_actions();
        let left_out = left.get_output_actions();
        let left_actions = left_in.union(&left_out).cloned().collect::<HashSet<_>>();

        let right_in = right.get_input_actions();
        let right_out = right.get_output_actions();
        let right_actions = right_in.union(&right_out).cloned().collect::<HashSet<_>>();

        if !left_out.is_disjoint(&right_out) {
            return ActionFailure::not_disjoint(
                (left.as_ref(), left_out),
                (right.as_ref(), right_out),
            )
            .map_err(|e| e.to_rfcomp(left, right));
        }

        // Act_i = Act1_i \ Act2_o ∪ Act2_i \ Act1_o
        let mut inputs = HashSet::new();

        for a in &left_in {
            if !right_out.contains(a) {
                inputs.insert(a.clone());
            }
        }
        for a in &right_in {
            if !left_out.contains(a) {
                inputs.insert(a.clone());
            }
        }

        // Act_o = Act1_o ∪ Act2_o
        let outputs = left_out.union(&right_out).cloned().collect();

        Ok(Box::new(Composition {
            left,
            right,
            inputs,
            outputs,
            left_unique_actions: left_actions.difference(&right_actions).cloned().collect(),
            right_unique_actions: right_actions.difference(&left_actions).cloned().collect(),
            common_actions: left_actions.intersection(&right_actions).cloned().collect(),
            dim,
        }))
    }
}

impl ComposedTransitionSystem for Composition {
    fn next_transitions(&self, location: Rc<LocationTree>, action: &str) -> Vec<Transition> {
        assert!(self.actions_contain(action));

        let loc_left = location.get_left();
        let loc_right = location.get_right();

        if self.common_actions.contains(action) {
            let left = self.left.next_transitions(loc_left, action);
            let right = self.right.next_transitions(loc_right, action);
            return Transition::combinations(&left, &right, CompositionType::Composition);
        }

        if self.left_unique_actions.contains(action) {
            let left = self.left.next_transitions(loc_left, action);
            return Transition::combinations(
                &left,
                &vec![Transition::without_id(loc_right, self.dim)],
                CompositionType::Composition,
            );
        }

        if self.right_unique_actions.contains(action) {
            let right = self.right.next_transitions(loc_right, action);
            return Transition::combinations(
                &vec![Transition::without_id(loc_left, self.dim)],
                &right,
                CompositionType::Composition,
            );
        }

        unreachable!()
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        (&self.left, &self.right)
    }

    fn get_children_mut(&mut self) -> (&mut TransitionSystemPtr, &mut TransitionSystemPtr) {
        (&mut self.left, &mut self.right)
    }

    fn get_composition_type(&self) -> CompositionType {
        CompositionType::Composition
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

    fn check_local_consistency(&self) -> crate::system::query_failures::ConsistencyResult {
        self.left.check_local_consistency()?;
        self.right.check_local_consistency()
    }
}
