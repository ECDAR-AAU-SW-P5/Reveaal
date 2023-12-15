use crate::model_objects::declarations::{DeclarationProvider, Declarations};
use crate::model_objects::{Component, State, Transition};
use crate::system::local_consistency::{self};
use crate::system::query_failures::{
    ActionFailure, ConsistencyResult, DeterminismResult, SystemRecipeFailure,
};
use crate::system::specifics::SpecificLocation;
use crate::transition_systems::clock_reduction::clock_removal::{
    rebuild_bounds, remove_clocks_from_federation, remove_clocks_from_location,
};
use crate::transition_systems::{LocationTree, TransitionSystem, TransitionSystemPtr};
use edbm::util::bounds::Bounds;
use edbm::util::constraints::ClockIndex;
use std::collections::hash_set::HashSet;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::rc::Rc;
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
    locations: HashMap<LocationID, Rc<LocationTree>>,
    location_edges: HashMap<LocationID, Vec<(Action, Transition)>>,
    initial_location: Option<Rc<LocationTree>>,
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

        let locations: HashMap<LocationID, Rc<LocationTree>> = component
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

    fn next_transitions(&self, locations: Rc<LocationTree>, action: &str) -> Vec<Transition> {
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

    fn get_initial_location(&self) -> Option<Rc<LocationTree>> {
        self.initial_location.clone()
    }

    fn get_all_locations(&self) -> Vec<Rc<LocationTree>> {
        self.locations.values().cloned().collect()
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

    fn get_location(&self, id: &LocationID) -> Option<Rc<LocationTree>> {
        self.locations.get(id).cloned()
    }

    fn component_names(&self) -> Vec<&str> {
        vec![&self.comp_info.name]
    }

    fn comp_infos(&'_ self) -> ComponentInfoTree<'_> {
        ComponentInfoTree::Info(&self.comp_info)
    }

    fn to_string(&self) -> String {
        self.comp_info.name.clone()
    }

    fn construct_location_tree(
        &self,
        target: SpecificLocation,
    ) -> Result<Rc<LocationTree>, String> {
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

    fn remove_clocks(
        &mut self,
        clocks: &[ClockIndex],
        shrink_expand_src: &[bool],
        shrink_expand_dst: &[bool],
    ) -> Result<(), String> {
        // Remove clocks from Declarations
        self.comp_info.declarations.remove_clocks(clocks);

        let shrink_expand_src = &shrink_expand_src.to_vec();
        let shrink_expand_dst = &shrink_expand_dst.to_vec();
        // Remove clocks from Locations
        for loc in self.locations.values_mut() {
            remove_clocks_from_location(loc, clocks, shrink_expand_src, shrink_expand_dst);
        }
        // Remove clocks from initial location
        if let Some(loc) = &mut self.initial_location {
            remove_clocks_from_location(loc, clocks, shrink_expand_src, shrink_expand_dst);
        }
        // Remove clocks from Edges
        for edge in self.location_edges.values_mut() {
            for (_, transition) in edge.iter_mut() {
                // Remove clocks from Guard
                transition.guard_zone = remove_clocks_from_federation(
                    transition.guard_zone.clone(),
                    clocks,
                    shrink_expand_src,
                    shrink_expand_dst,
                );

                // Remove clocks from Updates
                transition
                    .updates
                    .retain(|update| !clocks.contains(&update.clock_index));

                //move clocks to the left in Updates
                for update in &mut transition.updates {
                    let clocks_less = clocks.partition_point(|clock| clock < &update.clock_index);
                    update.clock_index -= clocks_less;
                }

                // Remove clocks from target locations (supposedly they're not updated when iterating self.locations)
                remove_clocks_from_location(
                    &mut transition.target_locations,
                    clocks,
                    shrink_expand_src,
                    shrink_expand_dst,
                );
            }
        }

        // Rebuild max bounds
        self.comp_info.max_bounds = rebuild_bounds(&self.comp_info.max_bounds, self.dim, clocks);

        self.dim -= clocks.len();

        Ok(())
    }
}
