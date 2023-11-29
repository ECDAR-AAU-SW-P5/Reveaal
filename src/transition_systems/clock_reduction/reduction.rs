use crate::system::query_failures::SystemRecipeFailure;
use crate::transition_systems::clock_reduction::clock_analysis_graph::find_redundant_clocks;
use crate::transition_systems::clock_reduction::clock_reduction_instruction::ClockReductionInstruction;
use crate::transition_systems::TransitionSystemPtr;
use edbm::util::constraints::ClockIndex;
use log::debug;
use std::collections::HashMap;

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
) -> Result<(), Box<SystemRecipeFailure>> {
    if *dim == 0 {
        return Ok(());
    } else if rhs.is_none() {
        return clock_reduce_single(lhs, dim, quotient_clock);
    }
    let rhs = rhs.unwrap();

    let (l_clocks, r_clocks) = filter_redundant_clocks(
        find_redundant_clocks(lhs),
        find_redundant_clocks(rhs),
        quotient_clock,
        lhs.get_dim(),
    );

    debug!("Clocks to be reduced: {l_clocks:?} + {r_clocks:?}");
    *dim -= l_clocks.iter().chain(r_clocks.iter()).count();
    debug!("New dimension: {dim}");

    let (l_remove_clocks, l_replace_clocks) = extract_remove_and_replace_from_instruction(l_clocks);
    let (r_remove_clocks, r_replace_clocks) = extract_remove_and_replace_from_instruction(r_clocks);
    if !l_remove_clocks.is_empty() {
        lhs.remove_clocks(&l_remove_clocks).unwrap();
    }
    if !l_replace_clocks.is_empty() {
        lhs.replace_clocks(&l_replace_clocks).unwrap();
    }
    if !r_remove_clocks.is_empty() {
        rhs.remove_clocks(&r_remove_clocks).unwrap();
    }
    if !r_replace_clocks.is_empty() {
        rhs.replace_clocks(&r_replace_clocks).unwrap();
    }

    Ok(())
}

/// Clock reduces a "single_expression", such as consistency
/// # Arguments
///
/// * `sys`: The [`SystemRecipe`] to clock reduce
/// * `dim`: the dimension of the system
/// * `quotient_clock`: The clock for the quotient (This is not reduced)
///
/// returns: Result<(), SystemRecipeFailure>
fn clock_reduce_single(
    sys: &mut TransitionSystemPtr,
    dim: &mut usize,
    quotient_clock: Option<ClockIndex>,
) -> Result<(), Box<SystemRecipeFailure>> {
    let mut clocks = find_redundant_clocks(sys);
    clocks.retain(|ins| ins.get_clock_index() != quotient_clock.unwrap_or_default());
    debug!("Clocks to be reduced: {clocks:?}");
    *dim -= clocks.len();
    debug!("New dimension: {dim}");
    let (remove_clocks, replace_clocks) = extract_remove_and_replace_from_instruction(clocks);
    if !remove_clocks.is_empty() {
        sys.remove_clocks(&remove_clocks).unwrap();
    }
    if !replace_clocks.is_empty() {
        sys.replace_clocks(&replace_clocks).unwrap();
    }
    Ok(())
}

//todo consider removing clockreductioninstruction
fn extract_remove_and_replace_from_instruction(
    instructions: Vec<ClockReductionInstruction>,
) -> (Vec<ClockIndex>, HashMap<ClockIndex, ClockIndex>) {
    let mut remove_clocks: Vec<ClockIndex> = Vec::new();
    let mut replace_clocks: HashMap<ClockIndex, ClockIndex> = HashMap::new();
    for instruction in instructions {
        match instruction {
            ClockReductionInstruction::RemoveClock { clock_index } => {
                remove_clocks.push(clock_index);
            }
            ClockReductionInstruction::ReplaceClock {
                clock_index,
                replacing_clock,
            } => {
                replace_clocks.insert(clock_index, replacing_clock);
            }
        }
    }
    (remove_clocks, replace_clocks)
}

fn filter_redundant_clocks(
    lhs: Vec<ClockReductionInstruction>,
    rhs: Vec<ClockReductionInstruction>,
    quotient_clock: Option<ClockIndex>,
    split_index: ClockIndex,
) -> (
    Vec<ClockReductionInstruction>,
    Vec<ClockReductionInstruction>,
) {
    fn get_unique_redundant_clocks<P: Fn(ClockIndex) -> bool>(
        l: Vec<ClockReductionInstruction>,
        r: Vec<ClockReductionInstruction>,
        quotient: ClockIndex,
        bound_predicate: P,
    ) -> Vec<ClockReductionInstruction> {
        l.into_iter()
            // Takes clock instructions that also occur in the rhs system
            // This is done because the lhs also finds the redundant clocks from the rhs,
            // so to ensure that it should be removed, we check if it occurs on both sides
            // which would mean it can be removed
            // e.g "A <= B", we can find clocks from B that are not used in A, so they are marked as remove
            .filter(|ins| r.contains(ins))
            // Takes all the clocks within the bounds of the given system
            // This is done to ensure that we don't try to remove a clock from the rhs system
            .filter(|ins| bound_predicate(ins.get_clock_index()))
            // Removes the quotient clock
            .filter(|ins| ins.get_clock_index() != quotient)
            .collect()
    }
    let quotient_clock = quotient_clock.unwrap_or_default();
    (
        get_unique_redundant_clocks(lhs.clone(), rhs.clone(), quotient_clock, |c| {
            c <= split_index
        }),
        get_unique_redundant_clocks(rhs, lhs, quotient_clock, |c| c > split_index),
    )
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
        use crate::transition_systems::clock_reduction::clock_reduction_instruction::ClockReductionInstruction;
        use crate::transition_systems::clock_reduction::reduction::clock_reduce;
        use crate::transition_systems::TransitionSystemPtr;
        use crate::{JsonProjectLoader, DEFAULT_SETTINGS};
        use edbm::util::constraints::ClockIndex;
        use std::collections::{HashMap, HashSet};
        use test_case::test_case;
        use ClockReductionInstruction::{RemoveClock, ReplaceClock};

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
            clock_reduce(&mut system, None, &mut dim, None).unwrap();

            // Assert
            assert_eq!(dim, 0, "After removing the clocks, the dim should be 0");
            assert_eq!(system.get_dim(), 1, "global clock still exists");
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
            clock_reduce(&mut left_ts, Some(&mut right_ts), &mut dim, None).unwrap();

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
            let clock_reduction_instruction = find_redundant_clocks(&transition_system);

            // Assert
            assert_eq!(
                clock_reduction_instruction.len(),
                1,
                "Only one instruction needed"
            );
            let clock_name_to_index = create_clock_name_to_index(&transition_system);
            assert!(
                match &clock_reduction_instruction[0] {
                    RemoveClock { .. } => false,
                    ReplaceClock {
                        clock_index,
                        replacing_clock,
                    } => {
                        assert_eq!(
                            clock_index,
                            clock_name_to_index.get("component1:x").unwrap(),
                            "Clock component2:x can be replaced by component1:x"
                        );
                        assert_eq!(
                            replacing_clock,
                            clock_name_to_index.get("component0:x").unwrap(),
                            "Clocks get replaced by component1:x"
                        );
                        true
                    }
                },
                "Clock reduction instruction is replace clocks"
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
            let clock_reduction_instruction = find_redundant_clocks(&transition_system);

            // Assert
            assert_eq!(
                clock_reduction_instruction.len(),
                1,
                "Only one instruction needed"
            );
            let clock_name_to_index = create_clock_name_to_index(&transition_system);
            assert!(
                match &clock_reduction_instruction[0] {
                    RemoveClock { .. } => false,
                    ReplaceClock {
                        clock_index,
                        replacing_clock,
                    } => {
                        assert_eq!(
                            clock_index,
                            clock_name_to_index.get("component1:y").unwrap(),
                            "Clock y can be replaced by x"
                        );
                        assert_eq!(
                            replacing_clock,
                            clock_name_to_index.get("component0:x").unwrap(),
                            "Clocks get replaced by x"
                        );
                        true
                    }
                },
                "Clock reduction instruction is replace clocks"
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
            let clock_reduction_instruction = find_redundant_clocks(&transition_system);

            // Assert
            assert_eq!(
                clock_reduction_instruction.len(),
                0,
                "No reduction is possible"
            );
        }

        #[test]
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
            clock_reduce(&mut compiled, None, &mut dimensions, None).unwrap();

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

        fn create_clock_name_to_index(
            transition_system: &TransitionSystemPtr,
        ) -> HashMap<String, ClockIndex> {
            let mut clock_name_to_index: HashMap<String, ClockIndex> = HashMap::new();

            for (i, declaration) in transition_system.get_all_system_decls().iter().enumerate() {
                for (clock_name, clock_index) in &declaration.clocks {
                    clock_name_to_index
                        .insert(format!("component{}:{}", i, clock_name), *clock_index);
                }
            }
            clock_name_to_index
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
        use crate::transition_systems::clock_reduction::clock_reduction_instruction::ClockReductionInstruction;
        use crate::transition_systems::{CompiledComponent, TransitionSystem, TransitionSystemPtr};
        use edbm::util::constraints::ClockIndex;
        use std::collections::HashMap;
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
            let instructions = find_redundant_clocks(&(compiled_component as TransitionSystemPtr));

            // Assert
            for instruction in instructions {
                assert!(
                    match instruction {
                        ClockReductionInstruction::RemoveClock { .. } => {
                            false
                        }
                        ClockReductionInstruction::ReplaceClock {
                            clock_index,
                            replacing_clock,
                        } => {
                            &replacing_clock == clock_index_x && &clock_index == clock_index_y
                                || &clock_index == clock_index_z
                        }
                    },
                    "failed to assert either replace_clock 3->1 or 2->1"
                )
            }
        }

        #[test]
        fn remove_duplicate_from_three_synced_clocks() {
            // Arrange
            let component = read_json_component(
                "samples/json/ClockReductionTest/RedundantClocks",
                "Component1",
            )
            .unwrap();

            let dim = component.declarations.clocks.len() + 1;
            let mut clock_reduced_compiled_component =
                CompiledComponent::compile(component, dim, &mut 0).unwrap();
            let decls = clock_reduced_compiled_component.get_component_decls();

            let target_clock = decls.get_clock_index_by_name("x").unwrap();
            let replace_clocks = HashMap::from([
                (*decls.get_clock_index_by_name("y").unwrap(), *target_clock),
                (*decls.get_clock_index_by_name("z").unwrap(), *target_clock),
            ]);

            // Act
            clock_reduced_compiled_component
                .replace_clocks(&replace_clocks)
                .expect("Couldn't replace clocks");

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
            let instructions = find_redundant_clocks(&(compiled_component as TransitionSystemPtr));

            // Assert
            find_clock_in_reduction_instruction(instructions, clock_index)
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
            let instructions = find_redundant_clocks(&(compiled_component as TransitionSystemPtr));

            // Assert
            find_clock_in_reduction_instruction(instructions, clock_index)
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
            compiled_component.remove_clocks(&[clock_index]).unwrap();

            // Assert
            assert!(!compiled_component.get_all_system_decls()[0]
                .clocks
                .contains_key(clock));
        }

        /// Assert that a [`vec<&ClockReductionInstruction>`] contains an instruction that `clock` should
        /// be removed.
        fn find_clock_in_reduction_instruction(
            redundant_clocks: Vec<ClockReductionInstruction>,
            clock: ClockIndex,
        ) {
            assert!(redundant_clocks
                .iter()
                .any(|instruction| match instruction {
                    ClockReductionInstruction::RemoveClock { clock_index } => {
                        *clock_index == clock
                    }
                    _ => false,
                }));
        }
    }
}
