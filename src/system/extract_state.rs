use std::rc::Rc;

use edbm::zones::OwnedFederation;
use itertools::Itertools;

use crate::edge_eval::constraint_applier::apply_constraints_to_state;
use crate::extract_system_rep::SystemRecipe;
use crate::model_objects::expressions::{BoolExpression, ComponentVariable, StateExpression};
use crate::model_objects::{Declarations, State};
use crate::transition_systems::{CompositionType, LocationID, LocationTree, TransitionSystemPtr};

/// This function takes a [`StateExpression`], the system recipe, and the transitionsystem -
/// to define a state from the [`StateExpression`] which has clocks and locations.
/// `state_query` is the part of the query that describes the location and the clock constraints of the state.
/// `machine` defines which operators is used to define the transistion system.
/// `system` is the transition system.
pub fn get_state(
    state_query: &StateExpression,
    recipe: &SystemRecipe,
    system: &TransitionSystemPtr,
) -> Result<State, String> {
    // Check that there are no duplicated names in the system recipe components
    let components = recipe.get_components();

    if let Some((c1, c2)) = components
        .iter()
        .cartesian_product(components.iter())
        .filter(|(&c1, &c2)| c1 != c2)
        .find(|(c1, c2)| c1.name == c2.name && c1.special_id == c2.special_id)
    {
        return Err(format!(
            "Ambiguous component name: {}[{:?}] and {}[{:?}] are indistinguishable",
            c1.name, c1.special_id, c2.name, c2.special_id
        ));
    }

    // Get the locations that are part of the state
    let mut locations = get_locations(state_query)?;
    // Deduplicate locations
    locations.dedup();
    // Check that there are no ambiguous locations
    if let Some((l1, l2)) = locations
        .iter()
        .cartesian_product(locations.iter())
        .filter(|(l1, l2)| **l1 != **l2)
        .find(|(l1, l2)| l1.component == l2.component && l1.special_id == l2.special_id)
    {
        // TODO: Support ambiguous target (and maybe even start) locations in the future e.g. "Comp1.L1 || Comp1.L2".
        return Err(format!(
            "Ambiguous location: {} and {} refer to the same component",
            l1, l2
        ));
    }

    let loc_tree = construct_location_tree(&locations, recipe, system)?;

    let zone =
        create_zone_given_constraints(&state_query.to_bool_expression(&components)?, system)?;

    Ok(State::new(loc_tree, zone))
}

fn get_locations(expr: &StateExpression) -> Result<Vec<ComponentVariable>, String> {
    // We don't currently support states with disjunctions of locations.
    // TODO: Add support for disjunctions of locations.
    match expr {
        StateExpression::AND(exprs) => {
            let mut res = Vec::new();
            for expr in exprs {
                res.append(&mut get_locations(expr)?);
            }
            Ok(res)
        }
        StateExpression::OR(exprs) => {
            let mut res = Vec::new();
            for expr in exprs {
                res.append(&mut get_locations(expr)?);
            }
            if res.len() != 1 {
                return Err(format!(
                    "We do not support disjunctions with more than one location: {:?}",
                    expr
                ));
            }
            Ok(res)
        }
        StateExpression::Location(loc) => Ok(vec![loc.clone()]),
        StateExpression::NOT(expr) => {
            if !get_locations(expr)?.is_empty() {
                Err(format!(
                    "We do not support negations of locations: {:?}",
                    expr
                ))
            } else {
                Ok(Vec::new())
            }
        }
        _ => Ok(Vec::new()),
    }
}

fn create_zone_given_constraints(
    constraints: &BoolExpression,
    system: &TransitionSystemPtr,
) -> Result<OwnedFederation, String> {
    let fed = OwnedFederation::universe(system.get_dim());
    let unused_decl = Declarations::empty();
    apply_constraints_to_state(constraints, &unused_decl, fed)
}

fn construct_location_tree(
    locations: &Vec<ComponentVariable>,
    machine: &SystemRecipe,
    system: &TransitionSystemPtr,
) -> Result<Rc<LocationTree>, String> {
    match machine {
        SystemRecipe::Composition(left, right) => {
            let (left_system, right_system) = system.get_children();
            Ok(LocationTree::compose(
                construct_location_tree(locations, left, left_system)?,
                construct_location_tree(locations, right, right_system)?,
                CompositionType::Composition,
            ))
        }
        SystemRecipe::Conjunction(left, right) => {
            let (left_system, right_system) = system.get_children();
            Ok(LocationTree::compose(
                construct_location_tree(locations, left, left_system)?,
                construct_location_tree(locations, right, right_system)?,
                CompositionType::Conjunction,
            ))
        }
        SystemRecipe::Quotient(left, right, ..) => {
            let (left_system, right_system) = system.get_children();
            Ok(LocationTree::merge_as_quotient(
                construct_location_tree(locations, left, left_system)?,
                construct_location_tree(locations, right, right_system)?,
            ))
        }
        SystemRecipe::Component(component) => {
            match locations.iter().find(|loc| {
                loc.component == component.name && loc.special_id == component.special_id
            }) {
                None => Ok(LocationTree::build_any_location_tree()),
                Some(var) => system
                    .get_location(&LocationID::Simple(var.variable.clone()))
                    .ok_or(format!(
                        "Location {:?} does not exist in the component",
                        var,
                    )),
            }
        }
    }
}
