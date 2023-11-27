use crate::model_objects::declarations::{DeclarationProvider, Declarations};
use crate::model_objects::{Component, State, Transition};
use crate::system::local_consistency::{self};
use crate::system::query_failures::{
    ActionFailure, ConsistencyResult, DeterminismResult, SystemRecipeFailure,
};
use crate::system::specifics::SpecificLocation;
use crate::transition_systems::clock_reduction::clock_removal::remove_clock_from_federation;
use crate::transition_systems::compiled_update::CompiledUpdate;
use crate::transition_systems::{LocationTree, TransitionSystem, TransitionSystemPtr};
use edbm::util::bounds::Bounds;
use edbm::util::constraints::ClockIndex;
use std::collections::hash_set::HashSet;
use std::collections::HashMap;
use std::iter::FromIterator;
use CompositionType::Simple;

use super::transition_system::ComponentInfoTree;
use super::{CompositionType, LocationID};

type Action = String;

#[derive(Clone)]
pub struct ComponentInfo {
    pub name: String,
    pub id: u32,
    pub declarations: Declarations,
    max_bounds: Bounds,
}

#[derive(Clone)]
pub struct CompiledComponent {
    inputs: HashSet<Action>,
    outputs: HashSet<Action>,
    locations: HashMap<LocationID, LocationTree>,
    location_edges: HashMap<LocationID, Vec<(Action, Transition)>>,
    initial_location: Option<LocationTree>,
    comp_info: ComponentInfo,
    dim: ClockIndex,
}

impl CompiledComponent {
    pub fn compile_with_actions(
        component: Component,
        inputs: HashSet<String>,
        outputs: HashSet<String>,
        dim: ClockIndex,
        id: u32,
    ) -> Result<Box<Self>, Box<SystemRecipeFailure>> {
        if !inputs.is_disjoint(&outputs) {
            ActionFailure::not_disjoint_io(&component.name, inputs.clone(), outputs.clone())
                .map_err(|e| e.to_simple_failure(&component.name))?;
        }

        let locations: HashMap<LocationID, LocationTree> = component
            .locations
            .iter()
            .map(|loc| {
                let loc = LocationTree::simple(loc, component.get_declarations(), dim);
                (loc.id.clone(), loc)
            })
            .collect();

        let mut location_edges: HashMap<LocationID, Vec<(Action, Transition)>> =
            locations.keys().map(|k| (k.clone(), vec![])).collect();

        log::debug!(
            "decl for {:?}: {:?}",
            component.name,
            component.declarations
        );
        log::debug!("Edges: {:?}", component.edges);
        for edge in &component.edges {
            let id = LocationID::Simple(edge.source_location.clone());
            let transition = Transition::from_component_and_edge(&component, edge, dim);
            location_edges
                .get_mut(&id)
                .unwrap()
                .push((edge.sync.clone(), transition));
        }

        let initial_location = locations.values().find(|loc| loc.is_initial()).cloned();

        let max_bounds = component.get_max_bounds(dim);
        Ok(Box::new(CompiledComponent {
            inputs,
            outputs,
            locations,
            location_edges,
            initial_location,
            dim,
            comp_info: ComponentInfo {
                name: component.name,
                declarations: component.declarations,
                max_bounds,
                id,
            },
        }))
    }

    pub fn compile(
        component: Component,
        dim: ClockIndex,
        component_index: &mut u32,
    ) -> Result<Box<Self>, Box<SystemRecipeFailure>> {
        let inputs = HashSet::from_iter(component.get_input_actions());
        let outputs = HashSet::from_iter(component.get_output_actions());
        let index = *component_index;
        *component_index += 1;
        Self::compile_with_actions(component, inputs, outputs, dim, index)
    }

    fn _comp_info(&self) -> &ComponentInfo {
        &self.comp_info
    }

    /// Should only ever be borrowed
    pub fn get_component_decls(&self) -> &Declarations {
        &self.comp_info.declarations
    }
}

impl TransitionSystem for CompiledComponent {
    fn get_local_max_bounds(&self, loc: &LocationTree) -> Bounds {
        if loc.is_universal() || loc.is_inconsistent() {
            Bounds::new(self.get_dim())
        } else {
            self.comp_info.max_bounds.clone()
        }
    }

    fn get_dim(&self) -> ClockIndex {
        self.dim
    }

    fn next_transitions(&self, locations: &LocationTree, action: &str) -> Vec<Transition> {
        assert!(self.actions_contain(action));
        let is_input = self.inputs_contain(action);

        if locations.is_universal() {
            return vec![Transition::without_id(locations, self.dim)];
        }

        if locations.is_inconsistent() && is_input {
            return vec![Transition::without_id(locations, self.dim)];
        }

        let mut transitions = vec![];
        let edges = self.location_edges.get(&locations.id).unwrap();

        for (channel, transition) in edges {
            if *channel == action {
                transitions.push(transition.clone());
            }
        }

        transitions
    }

    fn get_input_actions(&self) -> HashSet<String> {
        self.inputs.clone()
    }

    fn get_output_actions(&self) -> HashSet<String> {
        self.outputs.clone()
    }

    fn get_actions(&self) -> HashSet<String> {
        self.inputs.union(&self.outputs).cloned().collect()
    }

    fn get_initial_location(&self) -> Option<LocationTree> {
        self.initial_location.clone()
    }

    fn get_all_locations(&self) -> Vec<LocationTree> {
        self.locations.values().cloned().collect()
    }

    fn get_location(&self, id: &LocationID) -> Option<LocationTree> {
        self.locations.get(id).cloned()
    }

    fn get_all_system_decls(&self) -> Vec<&Declarations> {
        vec![&self.comp_info.declarations]
    }

    fn check_determinism(&self) -> DeterminismResult {
        local_consistency::check_determinism(self)
    }

    fn check_local_consistency(&self) -> ConsistencyResult {
        local_consistency::is_least_consistent(self)
    }

    fn get_initial_state(&self) -> Option<State> {
        let init_loc = self.get_initial_location()?;

        State::from_location(init_loc, self.dim)
    }

    fn get_children(&self) -> (&TransitionSystemPtr, &TransitionSystemPtr) {
        unreachable!()
    }

    fn get_composition_type(&self) -> CompositionType {
        Simple
    }

    fn comp_infos(&'_ self) -> ComponentInfoTree<'_> {
        ComponentInfoTree::Info(&self.comp_info)
    }

    fn to_string(&self) -> String {
        self.comp_info.name.clone()
    }

    fn component_names(&self) -> Vec<&str> {
        vec![&self.comp_info.name]
    }

    fn remove_clocks(&mut self, clocks: &Vec<ClockIndex>) -> Result<(), String> {
        // Remove clock from Declarations
        self.comp_info.declarations.remove_clocks(clocks);
        // Remove clock from Locations
        for loc in self.locations.values_mut() {
            // Remove from Invariant
            for clock in clocks {
                // todo: replace with remove_many
                match &loc.invariant {
                    None => {}
                    Some(inv) => {
                        loc.invariant = Some(remove_clock_from_federation(&inv, clock, None));
                    }
                }
            }
        }
        // Remove clock from Edges
        for edge in self.location_edges.values_mut() {
            for (_, transition) in edge.iter_mut() {
                // todo: replace with remove_many
                for clock in clocks {
                    // Remove clock from Guard
                    transition.guard_zone =
                        remove_clock_from_federation(&transition.guard_zone, clock, None);
                }

                // Remove clock from Update
                transition
                    .updates
                    .retain(|update| !clocks.contains(&update.clock_index));
            }
        }

        self.dim -= clocks.len();
        Ok(())
    }

    fn replace_clocks(&mut self, clocks: &HashMap<ClockIndex, ClockIndex>) -> Result<(), String> {
        // Replace clock from Declarations
        self.comp_info.declarations.replace_clocks(clocks);
        // Replace clock from Locations
        for loc in self.locations.values_mut() {
            // Replace from Invariant
            for (clock_to_be_replaced, new_clock) in clocks {
                // todo: replace with replace_many
                match &loc.invariant {
                    None => {}
                    Some(inv) => {
                        loc.invariant = Some(remove_clock_from_federation(
                            &inv,
                            clock_to_be_replaced,
                            Some(new_clock),
                        ));
                    }
                }
            }
        }
        // Replace clock from Edges
        for edge in self.location_edges.values_mut() {
            for (_, transition) in edge.iter_mut() {
                // todo: replace with replace_many
                for (clock_to_be_replaced, new_clock) in clocks {
                    // Replace clock from Guard
                    transition.guard_zone = remove_clock_from_federation(
                        &transition.guard_zone,
                        clock_to_be_replaced,
                        Some(new_clock),
                    );
                }

                // Replace clock from Updates in Edge
                transition.updates = transition
                    .updates
                    .iter()
                    .map(|update| match clocks.get(&update.clock_index) {
                        None => CompiledUpdate {
                            clock_index: update.clock_index,
                            value: update.value,
                        },
                        Some(clock) => CompiledUpdate {
                            clock_index: *clock,
                            value: update.value,
                        },
                    })
                    .collect();
            }
        }

        self.dim -= clocks.len();
        Ok(())
    }

    fn construct_location_tree(&self, target: SpecificLocation) -> Result<LocationTree, String> {
        match target {
            SpecificLocation::ComponentLocation { comp, location_id } => {
                assert_eq!(comp.name, self.comp_info.name);
                self.get_all_locations()
                    .into_iter()
                    .find(|loc| loc.id == LocationID::Simple(location_id.clone()))
                    .ok_or_else(|| {
                        format!(
                            "Could not find location {} in component {}",
                            location_id, self.comp_info.name
                        )
                    })
            }
            SpecificLocation::BranchLocation(_, _, _) | SpecificLocation::SpecialLocation(_) => {
                unreachable!("Should not happen at the level of a component.")
            }
        }
    }
}
