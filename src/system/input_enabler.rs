use edbm::zones::OwnedFederation;

use crate::edge_eval::constraint_applier;
use crate::model_objects::expressions::BoolExpression;
use crate::model_objects::{Component, DeclarationProvider, Edge, SyncType};

pub fn make_input_enabled(component: &mut Component, inputs: &[String]) {
    let dimension = component.declarations.get_clock_count() + 1;
    let mut new_edges: Vec<Edge> = vec![];
    let input_edges = component
        .edges
        .iter()
        .filter(|edge| edge.sync_type == SyncType::Input);
    for location in &component.locations {
        let mut location_inv_zone = OwnedFederation::universe(dimension);

        if let Some(invariant) = &location.invariant {
            location_inv_zone = constraint_applier::apply_constraints_to_state(
                invariant,
                component.get_declarations(),
                location_inv_zone,
            )
            .unwrap();
        }

        // No constraints on any clocks
        let full_federation = location_inv_zone.clone();
        let location_edges = input_edges
            .clone()
            .filter(|edge| edge.source_location == location.id);

        for input in inputs {
            let specific_edges = location_edges
                .clone()
                .filter(|edge| edge.sync == *input || edge.sync == "*");
            let mut zones_federation = OwnedFederation::empty(dimension);

            for edge in specific_edges {
                let mut guard_zone = OwnedFederation::universe(dimension);
                if let Some(target_invariant) = &component
                    .get_location_by_name(edge.target_location.as_str())
                    .invariant
                {
                    guard_zone = constraint_applier::apply_constraints_to_state(
                        target_invariant,
                        component.get_declarations(),
                        guard_zone,
                    )
                    .unwrap();
                }

                if let Some(updates) = &edge.update {
                    for update in updates {
                        let cu = update.compiled(component.get_declarations());
                        guard_zone = cu.apply_as_guard(guard_zone);
                        guard_zone = cu.apply_as_free(guard_zone);
                    }
                }

                if let Some(guard) = &edge.guard {
                    guard_zone = constraint_applier::apply_constraints_to_state(
                        guard,
                        component.get_declarations(),
                        guard_zone,
                    )
                    .unwrap();
                }

                zones_federation += guard_zone.intersection(&location_inv_zone);
            }

            let result_federation = full_federation.clone().subtraction(&zones_federation);

            if result_federation.is_empty() {
                continue;
            }

            new_edges.push(Edge {
                id: format!("input_{}_{}", location.id, input),
                source_location: location.id.to_string(),
                target_location: location.id.to_string(),
                sync_type: SyncType::Input,
                guard: BoolExpression::from_disjunction(
                    &result_federation.minimal_constraints(),
                    &component.get_declarations().clocks,
                ),
                update: None,
                sync: input.to_string(),
            });
        }
    }

    component.edges.append(&mut new_edges);
}
