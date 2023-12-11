use std::rc::Rc;

use crate::model_objects::{State, Transition};
use crate::transition_systems::TransitionSystemPtr;

/// Represent a decision in a any composition of components: In the current `state` [`State`] we have decided to take this `action` [`String`].
#[derive(Debug, Clone)]
pub struct Decision {
    pub state: State,
    pub action: String,
    pub transition: Option<Transition>,
    pub next_state: State,
}

impl Decision {
    /// Resolves a [`Decision`]: use the `action` in the `state` and return a [`Vec`] of the possible [`Decision`]s from the destination [`State`].
    ///
    /// # Panics
    /// Panics if the [`Decision`] leads to no new states or is ambiguous (leads to multiple new states)
    pub fn resolve(&self, system: &TransitionSystemPtr) -> Vec<Decision> {
        let transitions =
            system.next_transitions(Rc::clone(&self.state.decorated_locations), &self.action);
        let mut next_states: Vec<_> = transitions
            .into_iter()
            .filter_map(|transition| transition.use_transition_alt(&self.state))
            .collect();

        assert_ne!(next_states.len(), 0, "Decision leads to no new states");
        assert_eq!(
            next_states.len(),
            1,
            "Ambiguous decision leads to multiple new states"
        );

        let next_state = next_states.pop().unwrap();

        Decision::get_decisions_from_state(next_state, system)
    }

    pub fn from_state_transition(
        mut state: State,
        transition: &Transition,
        action: impl Into<String>,
    ) -> Option<Decision> {
        // Get the zone that is allowed by the transition
        let allowed = transition.get_allowed_federation();
        // Intersect the state zone with the allowed zone
        state.update_zone(|zone| zone.intersection(&allowed));
        // Check if the new state is empty
        if !state.ref_zone().is_empty() {
            let next_state = transition.use_transition_alt(&state).expect(
                "If the allowed zone is non-empty, the transition should lead to a non-empty state",
            );
            Some(Decision {
                state,
                action: action.into(),
                transition: Some(transition.to_owned()),
                next_state,
            })
        } else {
            None
        }
    }

    /// Get all possible [`Decision`]s from a [`State`]
    pub fn get_decisions_from_state(state: State, system: &TransitionSystemPtr) -> Vec<Decision> {
        let mut next_decisions = vec![];

        for action in system.get_actions() {
            let possible_transitions =
                system.next_transitions(Rc::clone(&state.decorated_locations), &action);
            for t in possible_transitions {
                if let Some(decision) = Decision::from_state_transition(state.clone(), &t, &action)
                {
                    next_decisions.push(decision);
                }
            }
        }

        next_decisions
    }

    /// Get all possible [`Decision`]s from the initial state of a [`TransitionSystemPtr`]
    pub fn get_initial_decisions(system: &TransitionSystemPtr) -> Vec<Decision> {
        Decision::get_decisions_from_state(
            system
                .get_initial_state()
                .expect("Expected system to have initial state"),
            system,
        )
    }
}
