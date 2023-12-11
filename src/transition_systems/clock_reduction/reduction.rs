use crate::transition_systems::clock_reduction::clock_analysis_graph::find_redundant_clocks;
use crate::transition_systems::TransitionSystemPtr;
use edbm::util::constraints::ClockIndex;
use log::debug;
use std::collections::HashSet;

/// Function for a "safer" clock reduction that handles both the dimension of the DBM and the quotient index if needed be
/// # Arguments
/// `lhs`: The (main) [`TransitionSystemPtr`] to clock reduce\n
/// `rhs`: An optional [`TransitionSystemPtr`] used for multiple operands (Refinement)\n
/// `dim`: A mutable reference to the DBMs dimension for updating\n
/// `quotient_clock`: The clock for the quotient (This is not reduced)
/// # Returns
/// A `Result` used if the [`TransitionSystemPtr`](s) fails
pub fn clock_reduce(
    lhs: &mut TransitionSystemPtr,
    rhs: Option<&mut TransitionSystemPtr>,
    dim: &mut usize,
    quotient_clock: Option<ClockIndex>,
) {
    let quotient_clock = quotient_clock.unwrap_or_default();
    if *dim == 0 {
        return;
    } else if rhs.is_none() {
        return clock_reduce_single(lhs, dim, quotient_clock);
    }

    let rhs = rhs.unwrap();

    let (mut remove, lhs_combine) = find_redundant_clocks(lhs);
    let (rhs_remove, rhs_combine) = find_redundant_clocks(rhs);

    //only remove and replace clocks that both sides agree on and dont remove quotient
    remove.retain(|clock| rhs_remove.contains(clock) && clock != &quotient_clock);
    let clock_group = lhs_combine
        .into_iter()
        //equivalent clock group
        .filter(|replace| rhs_combine.contains(replace))
        .filter_map(|replace| {
            let clock_group: HashSet<ClockIndex> = replace
                .iter()
                .filter(|replace| replace != &&quotient_clock)
                .copied()
                .collect::<HashSet<ClockIndex>>();
            // minimum 2 clocks to be combined into 1
            if clock_group.len() < 2 {
                return None;
            }
            return Some(clock_group);
        })
        .collect();

    let count = get_count(&remove, &clock_group);

    let shrink_expand_src = vec![true; *dim + 1];

    debug!("Clocks to be reduced: {count:?}");
    *dim -= remove.len(); //count;
    debug!("New dimension: {dim}");

    if !remove.is_empty() {
        let shrink_expand = create_shrink_expand(&remove, &lhs.get_dim());

        lhs.remove_clocks(&remove, &shrink_expand_src, &shrink_expand)
            .unwrap();
        rhs.remove_clocks(&remove, &shrink_expand_src, &shrink_expand)
            .unwrap();
    }
}

fn create_shrink_expand(clocks: &Vec<ClockIndex>, dim: &usize) -> Vec<bool> {
    let mut shrink_expand: Vec<bool> = vec![true; *dim];
    for clock in clocks {
        shrink_expand[*clock] = false;
    }
    shrink_expand
}

// length of remove_clocks + all the clocks in each clock group - amount of clock groups.
fn get_count(remove_clocks: &Vec<ClockIndex>, combine_clocks: &Vec<HashSet<ClockIndex>>) -> usize {
    remove_clocks.len()
        + combine_clocks
            .iter()
            .map(|group| group.iter().sum::<usize>())
            .sum::<usize>()
        - combine_clocks.len()
}

/// Clock reduces a "single_expression", such as consistency
/// # Arguments
///
/// * `sys`: The [`SystemRecipe`] to clock reduce
/// * `dim`: the dimension of the system
/// * `quotient_clock`: The clock for the quotient (This is not reduced)
///
/// returns: Result<(), SystemRecipeFailure>
fn clock_reduce_single(sys: &mut TransitionSystemPtr, dim: &mut usize, quotient_clock: ClockIndex) {
    let (mut remove_clocks, mut combine_clocks) = find_redundant_clocks(sys);

    if !remove_clocks.is_empty() {
        remove_clocks.remove(quotient_clock);
    }
    for clock_group in &mut combine_clocks {
        clock_group.remove(&quotient_clock);
    }

    let clock_count: usize = get_count(&remove_clocks, &combine_clocks);

    let shrink_expand_src = vec![true; *dim + 1];

    debug!("Clocks to be reduced: {clock_count:?}");
    *dim -= remove_clocks.len(); //clock_count;
    debug!("New dimension: {dim}");
    if !remove_clocks.is_empty() {
        let shrink_expand = create_shrink_expand(&remove_clocks, &sys.get_dim());

        sys.remove_clocks(&remove_clocks, &shrink_expand_src, &shrink_expand)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    mod transition_system {
        use crate::data_reader::json_reader::read_json_component;
        use crate::extract_system_rep::SystemRecipe;
        use crate::tests::refinement::helper::json_run_query;
        use crate::transition_systems::clock_reduction::clock_analysis_graph::{
            find_redundant_clocks, ClockAnalysisGraph,
        };
        use crate::transition_systems::clock_reduction::reduction::clock_reduce;
        use crate::transition_systems::TransitionSystemPtr;
        use crate::{JsonProjectLoader, DEFAULT_SETTINGS};
        use edbm::util::constraints::ClockIndex;
        use std::collections::HashSet;
        use test_case::test_case;

        const AG_PATH: &str = "samples/json/AG";

        #[test]
        fn component_with_no_used_clock() {
            // Arrange
            let comp = read_json_component(AG_PATH, "A").unwrap();

            let mut dim = comp.declarations.clocks.len();
            assert_eq!(dim, 4, "Component A should have 4 unused clocks");

            // Adding some extra big dimension to test that it is resized no matter what
            dim = 15;

            let mut component_index = 0;
            let mut system: TransitionSystemPtr = SystemRecipe::Component(Box::from(comp))
                .compile_with_index(dim, &mut component_index)
                .unwrap();

            // Act
            clock_reduce(&mut system, None, &mut dim, None);

            // Assert
            assert_eq!(dim, 1, "After removing the clocks, the dim should be 1");
            assert!(
                json_run_query(AG_PATH, "consistency: A").is_ok(),
                "Component A should be consistent"
            );
        }

        #[test]
        fn component_with_no_used_clock_in_system() {
            // Arrange
            let (lhs, rhs, mut dim) = get_two_components(AG_PATH, "A", "A");
            assert_eq!(dim, 8, "The components A & A has 8 unused clocks");

            let mut component_index = 0;
            let mut left_ts: TransitionSystemPtr =
                lhs.compile_with_index(dim, &mut component_index).unwrap();
            let mut right_ts: TransitionSystemPtr =
                rhs.compile_with_index(dim, &mut component_index).unwrap();

            // Act
            clock_reduce(&mut left_ts, Some(&mut right_ts), &mut dim, None);

            // Assert
            assert_eq!(dim, 0, "After removing the clocks, the dim should be 0");

            assert!(
                json_run_query(AG_PATH, "refinement: A <= A").is_ok(),
                "A should refine itself"
            );
        }

        #[test]
        fn same_component_clock_detection() {
            // Arrange
            let (sr_component1, sr_component2, dim) = get_two_components(
                "samples/json/ClockReductionTest/AdvancedClockReduction/Conjunction/SameComponent",
                "Component1",
                "Component1",
            );
            let system_recipe = SystemRecipe::Conjunction(sr_component1, sr_component2);
            let transition_system = system_recipe.compile(dim).unwrap();

            // Act
            let (remove_clocks, combine_clocks) = find_redundant_clocks(&transition_system);

            // Assert
            assert_eq!(remove_clocks.len(), 0, "no remove clocks");
            let all_decls = transition_system.get_all_system_decls();
            let clockgroup = HashSet::from([
                *all_decls[1].get_clock_index_by_name("x").unwrap(),
                *all_decls[0].get_clock_index_by_name("x").unwrap(),
            ]);
            assert_eq!(&clockgroup, combine_clocks.first().unwrap());
            assert_eq!(
                combine_clocks
                    .iter()
                    .filter(|group| group.len() > 1)
                    .count(),
                1,
                "there should only be one clock_group to be combined"
            );
        }

        #[test_case("samples/json/ClockReductionTest/AdvancedClockReduction/Conjunction/Example1"; "conjunction_example1")]
        #[test_case("samples/json/ClockReductionTest/AdvancedClockReduction/Conjunction/ConjunctionCyclic"; "conjunction_cyclical_component")]
        fn replace_clock(path: &str) {
            // Arrange
            let (sr_component1, sr_component2, dim) =
                get_two_components(path, "Component1", "Component2");
            let system_recipe = SystemRecipe::Conjunction(sr_component1, sr_component2);
            let transition_system = system_recipe.compile(dim).unwrap();

            // Act
            let (remove_clocks, combine_clocks) = find_redundant_clocks(&transition_system);

            // Assert
            assert_eq!(remove_clocks.len(), 0, "no remove clocks");
            let all_decls = transition_system.get_all_system_decls();
            let clockgroup = HashSet::from([
                *all_decls[1].get_clock_index_by_name("y").unwrap(),
                *all_decls[0].get_clock_index_by_name("x").unwrap(),
            ]);
            assert!(
                clockgroup.eq(combine_clocks.first().unwrap()),
                "clock y in component1 and clock x in component2 should be combined"
            );
            assert_eq!(
                combine_clocks
                    .iter()
                    .filter(|group| group.len() > 1)
                    .count(),
                1,
                "there should only be one clock_group to be combined"
            );
        }

        #[test]
        fn composition_cyclical_component() {
            // Arrange
            let (sr_component1, sr_component2, dimensions) = get_two_components("samples/json/ClockReductionTest/AdvancedClockReduction/Composition/CyclicOnlyOutput",
                                                                                "Component1",
                                                                                "Component2");
            let transition_system = SystemRecipe::Composition(sr_component1, sr_component2)
                .compile(dimensions)
                .unwrap();

            // Act
            let (remove_clocks, combine_clocks) = find_redundant_clocks(&transition_system);

            // Assert
            assert_eq!(remove_clocks.len(), 0, "No reduction is possible");
            assert_eq!(
                combine_clocks
                    .iter()
                    .filter(|combine_clock| combine_clock.len() > 1)
                    .count(),
                0,
                "no reduction is possible"
            );
        }

        #[test]
        #[ignore]
        fn remove_clock() {
            // Arrange
            let (sr_component1, sr_component2, mut dimensions) = get_two_components(
                "samples/json/ClockReductionTest/AdvancedClockReduction/Conjunction/Example1",
                "Component1",
                "Component2",
            );
            let system_recipe = SystemRecipe::Conjunction(sr_component1, sr_component2);
            let mut compiled = system_recipe.compile(dimensions).unwrap();

            // Act
            clock_reduce(&mut compiled, None, &mut dimensions, None);

            // Assert
            for location in compiled.get_all_locations() {
                assert!(location.invariant.is_none(), "Should contain no invariants")
            }

            let graph = ClockAnalysisGraph::from_system(&compiled);
            for edge in &graph.edges {
                match format!("{}->{}", edge.from, edge.to).as_str() {
                    "(L0&&L4)->(L1&&L5)" => {
                        assert_eq!(
                            edge.guard_dependencies.len(),
                            2,
                            "edge (L0&&L4)->(L1&&L5) should only have 1 guard dependency"
                        );
                        assert!(edge.guard_dependencies.is_subset(&HashSet::from([0, 1])));
                        assert_eq!(
                            edge.updates.len(),
                            0,
                            "(L0&&L4)->(L1&&L5) should have no updates"
                        );
                    }
                    "(L1&&L5)->(L2&&L6)" => {
                        assert_eq!(
                            edge.guard_dependencies.len(),
                            0,
                            "edge (L0&&L4)->(L1&&L5) should only have 2 guard dependency"
                        );
                        for update in &edge.updates {
                            assert_eq!(
                                update.clock_index, 1,
                                "edge (L0&&L4)->(L1&&L5) should only update clock 1"
                            );
                        }
                    }
                    "(L2&&L6)->(L3&&L7)" => {
                        assert_eq!(
                            edge.guard_dependencies.len(),
                            0,
                            "edge (L0&&L4)->(L1&&L5) should only have 1 guard dependency"
                        );
                        assert_eq!(
                            edge.updates.len(),
                            0,
                            "(L2&&L6)->(L3&&L7) should have no updates"
                        );
                    }
                    e => panic!("unknown edge {}", e),
                }
            }
        }

        fn get_two_components(
            path: &str,
            comp1: &str,
            comp2: &str,
        ) -> (Box<SystemRecipe>, Box<SystemRecipe>, ClockIndex) {
            let project_loader = JsonProjectLoader::new_loader(path, DEFAULT_SETTINGS);

            let mut component_loader = project_loader.to_comp_loader();

            let mut component1 = component_loader.get_component(comp1).unwrap().clone();
            let mut component2 = component_loader.get_component(comp2).unwrap().clone();

            let mut next_clock_index: usize = 0;
            component1.set_clock_indices(&mut next_clock_index);
            component2.set_clock_indices(&mut next_clock_index);

            let sr_component1 = Box::new(SystemRecipe::Component(Box::new(component1)));
            let sr_component2 = Box::new(SystemRecipe::Component(Box::new(component2)));
            (sr_component1, sr_component2, next_clock_index)
        }
    }
    mod component_clock_removal {
        use crate::data_reader::json_reader::read_json_component;
        use crate::system::input_enabler;
        use crate::transition_systems::clock_reduction::clock_analysis_graph::find_redundant_clocks;
        use crate::transition_systems::clock_reduction::reduction::create_shrink_expand;
        use crate::transition_systems::{CompiledComponent, TransitionSystem, TransitionSystemPtr};
        use std::collections::HashSet;
        use test_case::test_case;

        #[test]
        fn find_duplicate_from_three_synced_clocks() {
            // Arrange
            let expected_clocks = ["x".to_string(), "y".to_string(), "z".to_string()];
            let mut component = read_json_component(
                "samples/json/ClockReductionTest/RedundantClocks",
                "Component1",
            )
            .unwrap();

            let inputs = component.get_input_actions();
            input_enabler::make_input_enabled(&mut component, &inputs);

            let dim = component.declarations.clocks.len() + 1;

            let compiled_component =
                CompiledComponent::compile(component.clone(), dim, &mut 0).unwrap();
            let clock_index_x = component
                .declarations
                .get_clock_index_by_name(&expected_clocks[0])
                .unwrap();
            let clock_index_y = component
                .declarations
                .get_clock_index_by_name(&expected_clocks[1])
                .unwrap();
            let clock_index_z = component
                .declarations
                .get_clock_index_by_name(&expected_clocks[2])
                .unwrap();

            // Act
            let (remove_clocks, combine_clocks) =
                find_redundant_clocks(&(compiled_component as TransitionSystemPtr));

            // Assert
            assert_eq!(remove_clocks.len(), 0, "no remove clocks in this test");
            assert!(
                combine_clocks.first().unwrap().eq(&HashSet::from([
                    *clock_index_x,
                    *clock_index_y,
                    *clock_index_z
                ])),
                "clock 1, 2, 3 must be combined"
            );
        }

        #[test]
        #[ignore]
        fn remove_duplicate_from_three_synced_clocks() {
            // Arrange
            let component = read_json_component(
                "samples/json/ClockReductionTest/RedundantClocks",
                "Component1",
            )
            .unwrap();

            let dim = component.declarations.clocks.len() + 1;
            let clock_reduced_compiled_component =
                CompiledComponent::compile(component, dim, &mut 0).unwrap();
            /*let decls = clock_reduced_compiled_component.get_component_decls();

            let a = HashSet::from([
                *decls.get_clock_index_by_name("x").unwrap(),
                *decls.get_clock_index_by_name("y").unwrap(),
                *decls.get_clock_index_by_name("z").unwrap(),
            ]);
            let combine_clocks = Vec::from([a]);*/

            // Act
            //clock_reduced_compiled_component.combine_clocks(&combine_clocks);

            // Assert
            let decls = clock_reduced_compiled_component.get_all_system_decls()[0];
            assert_eq!(*decls.clocks.get("x").unwrap(), 1);
            assert_eq!(*decls.clocks.get("y").unwrap(), 1);
            assert_eq!(*decls.clocks.get("z").unwrap(), 1);
        }

        /// Loads the sample in `samples/json/ClockReductionTest/UnusedClockWithCycle` which contains
        /// unused clocks. It then tests that these clocks are located correctly.
        #[test_case("Component1", "x")]
        #[test_case("Component2", "z")]
        #[test_case("Component3", "j")]
        fn cycles_find_unused_clocks(component_name: &str, unused_clock: &str) {
            // Arrange
            let component = read_json_component(
                "samples/json/ClockReductionTest/UnusedClockWithCycle",
                component_name,
            )
            .unwrap();

            let dim = component.declarations.clocks.len() + 1;
            let compiled_component: Box<CompiledComponent> =
                CompiledComponent::compile(component, dim, &mut 0).unwrap();

            let clock_index = *compiled_component
                .get_component_decls()
                .get_clock_index_by_name(unused_clock)
                .unwrap();

            // Act
            let (remove_clocks, _) =
                find_redundant_clocks(&(compiled_component as TransitionSystemPtr));

            // Assert
            assert!(remove_clocks.contains(&clock_index));
        }

        /// Loads the sample in `samples/json/ClockReductionTest/UnusedClock` which contains
        /// unused clocks. It then tests that these clocks are located correctly.
        #[test_case("Component1", "x")]
        #[test_case("Component2", "i")]
        #[test_case("Component3", "c")]
        fn find_unused_clocks(component_name: &str, unused_clock: &str) {
            // Arrange
            let component = read_json_component(
                "samples/json/ClockReductionTest/UnusedClock",
                component_name,
            )
            .unwrap();

            let dim = component.declarations.clocks.len() + 1;
            let compiled_component: Box<CompiledComponent> =
                CompiledComponent::compile(component, dim, &mut 0).unwrap();

            let clock_index = *compiled_component
                .get_component_decls()
                .get_clock_index_by_name(unused_clock)
                .unwrap();

            // Act
            let (remove_clocks, _) =
                find_redundant_clocks(&(compiled_component as TransitionSystemPtr));

            // Assert
            assert!(remove_clocks.contains(&clock_index));
        }

        #[test_case("Component1", "x")]
        #[test_case("Component2", "i")]
        #[test_case("Component3", "c")]
        fn remove_unused_clocks(component_name: &str, clock: &str) {
            // Arrange
            let component = read_json_component(
                "samples/json/ClockReductionTest/UnusedClock",
                component_name,
            )
            .unwrap();
            let dim = component.declarations.clocks.len() + 1;
            let mut compiled_component: Box<CompiledComponent> =
                CompiledComponent::compile(component, dim, &mut 0).unwrap();

            let clock_index = *compiled_component
                .get_component_decls()
                .get_clock_index_by_name(clock)
                .unwrap();

            // Act
            let remove_clocks = Vec::from([clock_index]);
            let shrink_expand_dst = create_shrink_expand(&remove_clocks, &dim);
            let shrink_expand_src = vec![true; dim];

            compiled_component
                .remove_clocks(&remove_clocks, &shrink_expand_src, &shrink_expand_dst)
                .unwrap();

            // Assert
            assert!(!compiled_component.get_all_system_decls()[0]
                .clocks
                .contains_key(clock));
        }
    }
}
