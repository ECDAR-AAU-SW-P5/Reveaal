use super::ComponentInfo;
use super::{CompositionType, LocationID, LocationTree};
use crate::model_objects::declarations::Declarations;
use crate::model_objects::{Component, State, Transition};
use crate::parse_queries::parse_to_system_expr;
use crate::system::query_failures::{ConsistencyResult, DeterminismResult};
use crate::system::specifics::SpecificLocation;
use crate::{
    data_reader::component_loader::ComponentContainer, extract_system_rep::get_system_recipe,
    ComponentLoader,
};
use dyn_clone::{clone_trait_object, DynClone};
use edbm::util::{bounds::Bounds, constraints::ClockIndex};
use std::collections::hash_set::HashSet;

pub type TransitionSystemPtr = Box<dyn TransitionSystem>;
pub type Action = String;
pub type EdgeTuple = (Action, Transition);
pub type EdgeIndex = (LocationID, usize);

pub enum ComponentInfoTree<'a> {
    Info(&'a ComponentInfo),
    Composition(Box<ComponentInfoTree<'a>>, Box<ComponentInfoTree<'a>>),
}

impl<'a> ComponentInfoTree<'a> {
    pub fn iter(&'a self) -> Box<dyn Iterator<Item = &'a ComponentInfo> + '_> {
        match self {
            ComponentInfoTree::Info(info) => Box::new(std::iter::once(*info)),
            ComponentInfoTree::Composition(left, right) => {
                Box::new(left.iter().chain(right.iter()))
            }
        }
    }

    pub fn split(self) -> (ComponentInfoTree<'a>, ComponentInfoTree<'a>) {
        match self {
            ComponentInfoTree::Composition(left, right) => (*left, *right),
            ComponentInfoTree::Info(_) => {
                unreachable!("Cannot split a ComponentInfoTree with only one ComponentInfo")
            }
        }
    }

    pub fn info(&self) -> &ComponentInfo {
        match self {
            ComponentInfoTree::Info(info) => info,
            ComponentInfoTree::Composition(_, _) => {
                unreachable!(
                    "Cannot get info from a ComponentInfoTree with more than one ComponentInfo"
                )
            }
        }
    }
}
/// Equal to an abstract class in a composite pattern for a given transition system
/// A Transition system can be a single [crate::transition_systems::compiled_component] or multiple
/// put together as either a [crate::transition_systems::quotient] or a [crate::transition_systems::common::ComposedTransitionSystem]
///
/// A [ComposedTransitionSystem] can either be [Conjunction] or [Composition]
pub trait TransitionSystem: DynClone {
    fn get_local_max_bounds(&self, loc: &LocationTree) -> Bounds;
    fn get_dim(&self) -> ClockIndex;

    fn next_transitions_if_available(
        &self,
        location: &LocationTree,
        action: &str,
    ) -> Vec<Transition> {
        if self.actions_contain(action) {
            self.next_transitions(location, action)
        } else {
            vec![]
        }
    }

    fn next_transitions(&self, location: &LocationTree, action: &str) -> Vec<Transition>;

    fn next_outputs(&self, location: &LocationTree, action: &str) -> Vec<Transition> {
        debug_assert!(self.get_output_actions().contains(action));
        self.next_transitions(location, action)
    }

    fn next_inputs(&self, location: &LocationTree, action: &str) -> Vec<Transition> {
        debug_assert!(self.get_input_actions().contains(action));
        self.next_transitions(location, action)
    }

    fn get_input_actions(&self) -> HashSet<String>;

    fn inputs_contain(&self, action: &str) -> bool {
        self.get_input_actions().contains(action)
    }

    fn get_output_actions(&self) -> HashSet<String>;

    fn outputs_contain(&self, action: &str) -> bool {
        self.get_output_actions().contains(action)
    }

    fn get_actions(&self) -> HashSet<String>;

    fn actions_contain(&self, action: &str) -> bool {
        self.get_actions().contains(action)
    }

    fn get_initial_location(&self) -> Option<LocationTree>;

    /// Function to get all locations from a [`TransitionSystem`]
    /// #### Warning
    /// This function utilizes a lot of memory. Use with caution
    fn get_all_locations(&self) -> Vec<LocationTree>;

    fn get_location(&self, id: &LocationID) -> Option<LocationTree> {
        self.get_all_locations()
            .iter()
            .find(|loc| loc.id == *id)
            .cloned()
    }

    fn get_all_system_decls(&self) -> Vec<&Declarations>;

    fn precheck_sys_rep(&self) -> ConsistencyResult {
        self.check_determinism()?;
        self.check_local_consistency()
    }

    fn check_determinism(&self) -> DeterminismResult;

    fn check_local_consistency(&self) -> ConsistencyResult;

    fn get_initial_state(&self) -> Option<State>;

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr);

    fn get_composition_type(&self) -> CompositionType;

    fn comp_infos(&'_ self) -> ComponentInfoTree<'_> {
        let (left, right) = self.get_children();
        let left_info = left.comp_infos();
        let right_info = right.comp_infos();
        ComponentInfoTree::Composition(Box::new(left_info), Box::new(right_info))
    }

    fn to_string(&self) -> String {
        if self.get_composition_type() == CompositionType::Simple {
            panic!("Simple Transition Systems should implement to_string() themselves.")
        }
        let (left, right) = self.get_children();
        let comp = match self.get_composition_type() {
            CompositionType::Conjunction => "&&",
            CompositionType::Composition => "||",
            CompositionType::Quotient => r"\\",
            CompositionType::Simple => unreachable!(),
        };
        format!("({} {} {})", left.to_string(), comp, right.to_string())
    }

    /// Returns a [`Vec`] of all component names in a given [`TransitionSystem`].
    fn component_names(&self) -> Vec<&str> {
        let children = self.get_children();
        let left_child = children.0;
        let right_child = children.1;
        left_child
            .component_names()
            .into_iter()
            .chain(right_child.component_names())
            .collect()
    }

    /// Assumes clocks are in order such as [1, 3, 4, 5]
    fn remove_clocks(
        &mut self,
        clocks: &[ClockIndex],
        shrink_expand_src: &[bool],
        shrink_expand_dst: &[bool],
    ) -> Result<(), String>;

    fn construct_location_tree(&self, target: SpecificLocation) -> Result<LocationTree, String>;
}

/// Returns a [`TransitionSystemPtr`] equivalent to a `composition` of some `components`.
pub fn components_to_transition_system(
    components: Vec<Component>,
    composition: &str,
) -> TransitionSystemPtr {
    let mut component_container = ComponentContainer::from(components);
    component_loader_to_transition_system(&mut component_container, composition)
}

/// Returns a [`TransitionSystemPtr`] equivalent to a `composition` of some components in a [`ComponentLoader`].
pub fn component_loader_to_transition_system(
    loader: &mut dyn ComponentLoader,
    composition: &str,
) -> TransitionSystemPtr {
    let mut dimension = 0;
    let sys_expr = parse_to_system_expr(composition).unwrap();
    get_system_recipe(&sys_expr, loader, &mut dimension, &mut None)
        .unwrap()
        .compile(dimension)
        .unwrap()
}

clone_trait_object!(TransitionSystem);
