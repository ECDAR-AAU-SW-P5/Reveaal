#[cfg(test)]
pub mod clock_removal_tests {
    use crate::data_reader::json_reader::read_json_component;
    use crate::extract_system_rep::SystemRecipe;
    use crate::tests::refinement::helper::json_run_query;
    use crate::transition_systems::clock_reduction::reduction::clock_reduce;
    use crate::transition_systems::{CompiledComponent, TransitionSystem, TransitionSystemPtr};
    use std::collections::HashMap;
    use test_case::test_case;

    #[test_case("Component1", "x"; "Component1 x")]
    #[test_case("Component2", "i"; "Component2 i")]
    #[test_case("Component3", "c"; "Component3 c")]
    fn test_check_declarations_unused_clocks_are_removed(component_name: &str, clock: &str) {
        let component = read_json_component(
            "samples/json/ClockReductionTest/UnusedClock",
            component_name,
        )
        .unwrap();

        let clock_index = *component
            .declarations
            .get_clock_index_by_name(clock)
            .unwrap();

        let mut clock_reduced_compiled_component: TransitionSystemPtr = CompiledComponent::compile(
            component.clone(),
            component.declarations.clocks.len() + 1,
            &mut 0,
        )
        .unwrap();
        clock_reduced_compiled_component
            .remove_clocks(&vec![clock_index])
            .unwrap();

        let decls = clock_reduced_compiled_component.get_decls();

        assert!(!decls[0].clocks.contains_key(clock));
    }

    #[test]
    fn test_check_declarations_duplicated_clocks_are_removed() {
        let component = read_json_component(
            "samples/json/ClockReductionTest/RedundantClocks",
            "Component1",
        )
        .unwrap();

        let mut clock_reduced_compiled_component = CompiledComponent::compile(
            component.clone(),
            component.declarations.clocks.len() + 1,
            &mut 0,
        )
        .unwrap();
        let decls_vector = clock_reduced_compiled_component.get_decls();
        let decls = decls_vector.first().unwrap();

        let clock_1_index = decls.get_clock_index_by_name("x").unwrap();

        let mut replace_clocks = HashMap::new();
        replace_clocks.insert(*decls.get_clock_index_by_name("y").unwrap(), *clock_1_index);
        replace_clocks.insert(*decls.get_clock_index_by_name("z").unwrap(), *clock_1_index);

        clock_reduced_compiled_component
            .replace_clocks(&replace_clocks)
            .expect("Couldn't replace clocks");

        let decls = clock_reduced_compiled_component.get_decls();

        assert_eq!(*decls[0].clocks.get_key_value("x").unwrap().1, 1);
        assert_eq!(*decls[0].clocks.get_key_value("y").unwrap().1, 1);
        assert_eq!(*decls[0].clocks.get_key_value("z").unwrap().1, 1);
    }

    #[test]
    fn test_no_used_clock() {
        const PATH: &str = "samples/json/AG";

        let comp = read_json_component(PATH, "A").unwrap();

        let mut dim = comp.declarations.clocks.len();
        assert_eq!(
            dim, 4,
            "As of writing these tests, this component has 4 unused clocks"
        );
        let mut component_index = 0;
        let mut recipe: TransitionSystemPtr = SystemRecipe::Component(Box::from(comp))
            .compile_with_index(dim, &mut component_index)
            .unwrap();
        clock_reduce(&mut recipe, None, &mut dim, None).unwrap();
        assert_eq!(dim, 0, "After removing the clocks, the dim should be 0");

        assert!(
            json_run_query(PATH, "consistency: A").is_ok(),
            "A should be consistent"
        );
    }

    #[test]
    fn test_no_used_clock_multi() {
        const PATH: &str = "samples/json/AG";
        let mut dim = 0;
        let mut lhs = read_json_component(PATH, "A").unwrap();
        lhs.set_clock_indices(&mut dim);
        let mut rhs = read_json_component(PATH, "A").unwrap();
        rhs.set_clock_indices(&mut dim);

        assert_eq!(
            dim, 8,
            "As of writing these tests, these component has 8 unused clocks"
        );
        assert_eq!(
            lhs.declarations.clocks.len() + rhs.declarations.clocks.len(),
            8
        );

        let mut component_index = 0;
        let mut left_ts: TransitionSystemPtr = SystemRecipe::Component(Box::from(lhs))
            .compile_with_index(dim, &mut component_index)
            .unwrap();
        let mut right_ts: TransitionSystemPtr = SystemRecipe::Component(Box::from(rhs))
            .compile_with_index(dim, &mut component_index)
            .unwrap();

        clock_reduce(&mut left_ts, Some(&mut right_ts), &mut dim, None).unwrap();
        assert_eq!(dim, 0, "After removing the clocks, the dim should be 0");

        assert!(
            json_run_query(PATH, "refinement: A <= A").is_ok(),
            "A should refine itself"
        );
    }
}
