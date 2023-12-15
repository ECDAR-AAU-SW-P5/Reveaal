use edbm::util::constraints::ClockIndex;
use edbm::zones::OwnedFederation;
use log::{debug, trace};

use crate::edge_eval::constraint_applier::apply_constraints_to_state;
use crate::model_objects::expressions::BoolExpression;
use crate::model_objects::{
    Component, DeclarationProvider, Declarations, Edge, Location, SyncType,
};
use crate::system::save_component::combine_components;
use crate::transition_systems::TransitionSystemPtr;
use crate::transition_systems::{CompiledComponent, LocationTree};
use std::collections::{HashMap, HashSet};

use super::save_component::PruningStrategy;

pub fn prune_system(ts: TransitionSystemPtr, dim: ClockIndex) -> TransitionSystemPtr {
    ts.precheck_sys_rep()
        .expect("Cannot prune transitions system which is not least consistent");

    let inputs = ts.get_input_actions();
    let outputs = ts.get_output_actions();
    let comp = combine_components(&ts, PruningStrategy::NoPruning);

    let mut input_map: HashMap<String, Vec<String>> = HashMap::new();
    input_map.insert(comp.name.clone(), inputs.iter().cloned().collect());

    let result = prune(&comp, dim, inputs, outputs);

    result.unwrap()
}

struct PruneContext {
    comp: Component,
    inconsistent_locs: Vec<String>,
    inconsistent_parts: HashMap<String, OwnedFederation>,
    passed_pairs: Vec<(String, OwnedFederation)>,
    dim: ClockIndex,
}

impl PruneContext {
    fn decl(&self) -> &Declarations {
        self.comp.get_declarations()
    }

    fn get_loc(&self, name: &str) -> &Location {
        self.comp.get_location_by_name(name)
    }

    fn try_get_incons(&self, name: &str) -> Option<&OwnedFederation> {
        self.inconsistent_parts.get(name)
    }

    fn get_incons(&self, name: &str) -> &OwnedFederation {
        self.inconsistent_parts.get(name).unwrap()
    }

    fn remove_edge(&mut self, edge: &Edge) {
        if let Some(index) = self.comp.edges.iter().position(|e| *e == *edge) {
            trace!("Removing {}", edge);
            self.comp.edges.remove(index);
        }
    }

    fn update_edge_guard(&mut self, edge: &Edge, guard_fed: &OwnedFederation) {
        if let Some(index) = self.comp.edges.iter().position(|e| *e == *edge) {
            let guard = BoolExpression::from_disjunction(
                &guard_fed.minimal_constraints(),
                &self.decl().clocks,
            );

            trace!(
                "Updating {} with guard {}",
                edge,
                guard.as_ref().unwrap_or(&BoolExpression::Bool(true))
            );
            self.comp.edges.get_mut(index).unwrap().guard = guard;
        }
    }

    fn finish(self) -> (Component, HashMap<String, OwnedFederation>) {
        (self.comp, self.inconsistent_parts)
    }
}

pub fn prune(
    comp: &Component,
    dim: ClockIndex,
    inputs: HashSet<String>,
    outputs: HashSet<String>,
) -> Result<Box<CompiledComponent>, String> {
    let new_comp = comp.clone();
    let inconsistent_locs: Vec<_> = new_comp
        .locations
        .iter()
        .filter(|l| is_immediately_inconsistent(l, comp, dim))
        .map(|l| l.id.clone())
        .collect();
    let inconsistent_parts: HashMap<String, OwnedFederation> = inconsistent_locs
        .iter()
        .map(|id| (id.clone(), OwnedFederation::universe(dim)))
        .collect();

    trace!("Inconsistent locs: {:?}", inconsistent_locs);

    let mut context = PruneContext {
        comp: new_comp,
        inconsistent_locs,
        inconsistent_parts,
        passed_pairs: vec![],
        dim,
    };

    while let Some(target_loc) = context.inconsistent_locs.pop() {
        // TODO: If is initial

        //Handle edges
        for edge in comp
            .edges
            .iter()
            .filter(|e| e.target_location == target_loc)
        {
            if edge.sync_type == SyncType::Input {
                handle_input(edge, &mut context);
            }
            handle_output(edge, &mut context);
        }

        trace!(
            "Step {}",
            context
                .inconsistent_parts
                .iter()
                .map(|(loc, fed)| format!("{loc}: {fed}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let (mut new_comp, incons_parts) = context.finish();
    add_inconsistent_parts_to_invariants(&mut new_comp, incons_parts, dim);

    debug!(
        "Pruned component from {} edges to {} edges",
        comp.edges.len(),
        new_comp.edges.len()
    );

    CompiledComponent::compile_with_actions(new_comp, inputs, outputs, dim, 0)
        .map_err(|e| format!("Pruning failed: {}", e))
}

fn add_inconsistent_parts_to_invariants(
    comp: &mut Component,
    incons_parts: HashMap<String, OwnedFederation>,
    dim: ClockIndex,
) {
    let decls = comp.get_declarations().clone();
    for location in &mut comp.locations {
        if let Some(incons) = incons_parts.get(&location.id) {
            // get invariant
            let mut invariant_fed = OwnedFederation::universe(dim);
            if let Some(inv) = &location.invariant {
                invariant_fed = apply_constraints_to_state(inv, &decls, invariant_fed).unwrap();
            }
            // Remove inconsistent part
            invariant_fed = invariant_fed.subtraction(incons);
            // Set the new invariant
            location.invariant = BoolExpression::from_disjunction(
                &invariant_fed.minimal_constraints(),
                &decls.clocks,
            );
        }
    }
}

fn handle_input(edge: &Edge, context: &mut PruneContext) {
    let target_loc = context.get_loc(&edge.target_location).clone();
    let mut inconsistent_part = context.get_incons(&target_loc.id).clone();

    // apply target invariant
    if let Some(inv) = &target_loc.invariant {
        inconsistent_part =
            apply_constraints_to_state(inv, context.decl(), inconsistent_part).unwrap();
    }
    // apply updates as guard
    if let Some(updates) = &edge.update {
        for update in updates {
            inconsistent_part = update
                .compiled(context.decl())
                .apply_as_guard(inconsistent_part);
        }
    }

    if inconsistent_part.is_empty() {
        return;
    }

    let inconsistent_part = back_exploration_on_transition(edge, inconsistent_part, context);

    if !inconsistent_part.is_empty() {
        // in the next step, we need to check whether there is output transitions that could lead us away from the inconsistent state
        // such a transition needs to
        // a) have the same source as e
        // b) not be a selfloop
        // c) be an output
        // d) not lead to the inconsistent part of a state itself
        let copy = inconsistent_part.clone();
        let mut inconsistent_part = predt_of_all_outputs(&target_loc, inconsistent_part, context);

        if copy.equals(&inconsistent_part) {
            inconsistent_part = inconsistent_part.down();
            // apply source invariant

            let source_loc = context.get_loc(&edge.source_location);
            if let Some(inv) = &source_loc.invariant {
                inconsistent_part =
                    apply_constraints_to_state(inv, context.decl(), inconsistent_part).unwrap();
            }
        }

        process_source_location(&edge.source_location, inconsistent_part, context);
    }

    remove_transition_if_unsat(edge, context);
}

fn remove_transition_if_unsat(edge: &Edge, context: &mut PruneContext) {
    let mut edge_fed = OwnedFederation::universe(context.dim);
    // apply target invariant
    let target_loc = context.get_loc(&edge.target_location);
    if let Some(inv) = &target_loc.invariant {
        edge_fed = apply_constraints_to_state(inv, context.decl(), edge_fed).unwrap();
    }

    // Subtract target inconsistent part
    if let Some(incons) = context.try_get_incons(&edge.target_location) {
        edge_fed = edge_fed.subtraction(incons);
    }

    if let Some(updates) = &edge.update {
        // apply updates as guard
        for update in updates {
            edge_fed = update.compiled(context.decl()).apply_as_guard(edge_fed);
        }

        // apply updates as free
        if !edge_fed.is_empty() {
            for update in updates {
                edge_fed = update.compiled(context.decl()).apply_as_free(edge_fed);
            }
        }
    }

    // Apply guards
    if let Some(guard) = &edge.guard {
        edge_fed = apply_constraints_to_state(guard, context.decl(), edge_fed).unwrap();
    }

    // Apply source invariant
    let source_loc = context.get_loc(&edge.source_location);
    if let Some(inv) = &source_loc.invariant {
        edge_fed = apply_constraints_to_state(inv, context.decl(), edge_fed).unwrap();
    }

    // Subtract source inconsistent part
    if let Some(incons) = context.try_get_incons(&edge.source_location) {
        edge_fed = edge_fed.subtraction(incons);
    }

    if edge_fed.is_empty() {
        context.remove_edge(edge);
    }
}

fn process_source_location(
    source_loc: &String,
    mut inconsistent_part: OwnedFederation,
    context: &mut PruneContext,
) {
    if !inconsistent_part.is_empty() {
        if let Some(part) = context.inconsistent_parts.get(source_loc) {
            inconsistent_part = inconsistent_part.union(part);
        };

        context
            .inconsistent_parts
            .insert(source_loc.clone(), inconsistent_part.clone());

        if !context
            .passed_pairs
            .iter()
            .any(|(id, fed)| *id == *source_loc && fed.equals(&inconsistent_part))
        // Can this be fed.subset_eq(...) instead? it would be ~twice as fast
        {
            context
                .passed_pairs
                .push((source_loc.clone(), inconsistent_part));
            context.inconsistent_locs.push(source_loc.clone());
        }
    }
}

fn predt_of_all_outputs(
    source_loc: &Location,
    inconsistent_part: OwnedFederation,
    context: &mut PruneContext,
) -> OwnedFederation {
    let mut incons_fed = inconsistent_part;
    for other_edge in context
        .comp
        .edges
        .iter()
        .filter(|e| e.source_location == source_loc.id && e.sync_type == SyncType::Output)
    {
        let target_loc = context.get_loc(&other_edge.target_location);

        let mut saving_fed = OwnedFederation::universe(context.dim);
        // apply target invariant
        if let Some(inv) = &target_loc.invariant {
            saving_fed = apply_constraints_to_state(inv, context.decl(), saving_fed).unwrap();
        }

        // remove the parts of the target transition that are inconsistent.
        if context
            .inconsistent_locs
            .iter()
            .any(|id| *id == target_loc.id)
        {
            saving_fed = saving_fed.subtraction(context.get_incons(&other_edge.target_location))
        }

        // apply updates via free
        if !saving_fed.is_empty() {
            if let Some(updates) = &other_edge.update {
                for update in updates {
                    saving_fed = update.compiled(context.decl()).apply_as_free(saving_fed);
                }
            }
        }

        // apply edge guard
        if let Some(guard) = &other_edge.guard {
            saving_fed = apply_constraints_to_state(guard, context.decl(), saving_fed).unwrap();
        }

        // apply source invariant
        if let Some(inv) = &source_loc.invariant {
            saving_fed = apply_constraints_to_state(inv, context.decl(), saving_fed).unwrap();
        }

        // do temporal predecessor avoiding saving fed
        let predt_fed = incons_fed.predt(&saving_fed);

        incons_fed += predt_fed;
    }

    incons_fed
}

fn back_exploration_on_transition(
    edge: &Edge,
    mut inconsistent_part: OwnedFederation,
    context: &mut PruneContext,
) -> OwnedFederation {
    // apply updates via free
    if let Some(updates) = &edge.update {
        for update in updates {
            inconsistent_part = update
                .compiled(context.decl())
                .apply_as_free(inconsistent_part);
        }
    }

    // apply edge guard
    if let Some(guard) = &edge.guard {
        inconsistent_part =
            apply_constraints_to_state(guard, context.decl(), inconsistent_part).unwrap();
    }

    // apply source invariant
    let source = context.get_loc(&edge.source_location);
    if let Some(inv) = &source.invariant {
        inconsistent_part =
            apply_constraints_to_state(inv, context.decl(), inconsistent_part).unwrap();
    }

    inconsistent_part
}

fn handle_output(edge: &Edge, context: &mut PruneContext) {
    let target_incons = context.get_incons(&edge.target_location);
    if target_incons.is_universe() {
        // Fully inconsistent target
        context.remove_edge(edge);
    } else {
        // Partially inconsistent target
        let mut incons_after_reset = target_incons.clone();
        if let Some(updates) = &edge.update {
            // TODO: this is different from J-ecdar
            // apply updates as guard
            for update in updates {
                incons_after_reset = update
                    .compiled(context.decl())
                    .apply_as_guard(incons_after_reset);
            }

            // apply updates as free
            if !incons_after_reset.is_empty() {
                for update in updates {
                    incons_after_reset = update
                        .compiled(context.decl())
                        .apply_as_free(incons_after_reset);
                }
            }
        }
        let mut guard_fed = OwnedFederation::universe(context.dim);
        // Apply guards
        if let Some(guard) = &edge.guard {
            guard_fed = apply_constraints_to_state(guard, context.decl(), guard_fed).unwrap();
        }
        guard_fed = guard_fed.subtraction(&incons_after_reset);

        if guard_fed.is_empty() {
            context.remove_edge(edge);
        } else {
            context.update_edge_guard(edge, &guard_fed);
        }
    }

    // get source invariant
    let mut source_invariant = OwnedFederation::universe(context.dim);
    let source_loc = context.get_loc(&edge.source_location);
    if let Some(inv) = &source_loc.invariant {
        source_invariant =
            apply_constraints_to_state(inv, context.decl(), source_invariant).unwrap();
    }

    if source_invariant.can_delay_indefinitely() {
        // Source is not inconsistent, nothing more to do
    } else {
        let mut fed_that_saves_us = OwnedFederation::empty(context.dim);
        for other_edge in context.comp.edges.iter().filter(|e| {
            e.source_location == edge.source_location && e.sync_type == SyncType::Output
        }) {
            // calculate and backtrack the part that is NOT inconsistent

            // get target invariant
            let mut good_part = OwnedFederation::universe(context.dim);
            let target_loc = context.get_loc(&other_edge.target_location);
            if let Some(inv) = &target_loc.invariant {
                good_part = apply_constraints_to_state(inv, context.decl(), good_part).unwrap();
            }

            // If target is inconsistent we must avoid that part
            if let Some(incons) = context.try_get_incons(&other_edge.target_location) {
                good_part = good_part.subtraction(incons);
            }

            if let Some(updates) = &other_edge.update {
                // TODO: this is different from J-ecdar
                // apply updates as guard
                for update in updates {
                    good_part = update.compiled(context.decl()).apply_as_guard(good_part);
                }

                // apply updates as free
                if !good_part.is_empty() {
                    for update in updates {
                        good_part = update.compiled(context.decl()).apply_as_free(good_part);
                    }
                }
            }

            // Apply guards
            if let Some(guard) = &other_edge.guard {
                good_part = apply_constraints_to_state(guard, context.decl(), good_part).unwrap();
            }
            // We are allowed to delay into outputs
            good_part = good_part.down().intersection(&source_invariant);

            fed_that_saves_us += good_part;
        }

        let new_incon_part = source_invariant.subtraction(&fed_that_saves_us);
        process_source_location(&edge.source_location, new_incon_part, context)
    }
}

fn is_immediately_inconsistent(
    location: &Location,
    comp: &Component,
    dimensions: ClockIndex,
) -> bool {
    let loc = LocationTree::simple(location, &comp.declarations, dimensions);

    loc.is_inconsistent()

    /*
    let fed = loc.get_invariants();
    let res = match fed {
        Some(fed) => !fed.can_delay_indefinitely(),
        None => false,
    };
    if res {
        log::warn!(
            "loc: {} inv: {} inconsistent",
            loc.id,
            loc.get_invariants().unwrap()
        )
    }

    res
     */
}
