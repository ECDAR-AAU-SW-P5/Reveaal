#[cfg(test)]
pub mod clock_removal_tests {
    use crate::data_reader::json_reader::read_json_component;
    use crate::extract_system_rep::{clock_reduction, SystemRecipe};
    use crate::model_objects::Component;
    use crate::tests::refinement::helper::json_run_query;
    use crate::transition_systems::{CompiledComponent, TransitionSystem};
    use std::collections::HashSet;

    #[test]
    fn test_check_declarations_unused_clocks_are_removed() {
        check_declarations_unused_clocks_are_removed("Component1", "x");
        check_declarations_unused_clocks_are_removed("Component2", "i");
        check_declarations_unused_clocks_are_removed("Component3", "c");
    }

    impl Component {
        fn fit_decls(&mut self, index: edbm::util::constraints::ClockIndex) {
            self.declarations
                .clocks
                .values_mut()
                .filter(|val| **val > index)
                .for_each(|val| *val -= 1);
        }
    }

    fn check_declarations_unused_clocks_are_removed(component_name: &str, clock: &str) {
        let mut component = read_json_component(
            "samples/json/ClockReductionTest/UnusedClock",
            component_name,
        )
        .unwrap();

        let clock_index = *component
            .declarations
            .get_clock_index_by_name(clock)
            .unwrap();

        component.remove_clock(clock_index);
        component.fit_decls(clock_index);

        let clock_reduced_compiled_component = CompiledComponent::compile(
            component.clone(),
            component.declarations.clocks.len() + 1,
            &mut 0,
        )
        .unwrap();

        let decls = clock_reduced_compiled_component.get_decls();

        assert!(!decls[0].clocks.contains_key(clock));
    }

    #[test]
    fn test_check_declarations_duplicated_clocks_are_removed() {
        let mut component = read_json_component(
            "samples/json/ClockReductionTest/RedundantClocks",
            "Component1",
        )
        .unwrap();

        let clock_1_index = component.declarations.get_clock_index_by_name("x").unwrap();
        let mut duplicate_clocks_index = HashSet::new();
        duplicate_clocks_index
            .insert(*component.declarations.get_clock_index_by_name("y").unwrap());
        duplicate_clocks_index
            .insert(*component.declarations.get_clock_index_by_name("z").unwrap());

        component.replace_clock(*clock_1_index, &duplicate_clocks_index);

        let clock_reduced_compiled_component = CompiledComponent::compile(
            component.clone(),
            component.declarations.clocks.len() + 1,
            &mut 0,
        )
        .unwrap();

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

        let recipe = SystemRecipe::Component(Box::from(comp));
        clock_reduction::clock_reduce(&mut Box::from(recipe), None, &mut dim, None).unwrap();
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

        let l = SystemRecipe::Component(Box::from(lhs));
        let r = SystemRecipe::Component(Box::from(rhs));
        clock_reduction::clock_reduce(&mut Box::from(l), Some(&mut Box::from(r)), &mut dim, None)
            .unwrap();
        assert_eq!(dim, 0, "After removing the clocks, the dim should be 0");

        assert!(
            json_run_query(PATH, "refinement: A <= A").is_ok(),
            "A should refine itself"
        );
    }
}
