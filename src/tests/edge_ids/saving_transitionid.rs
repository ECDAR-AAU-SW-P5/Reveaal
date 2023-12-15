#[cfg(test)]
mod saving_transition_id_test {
    use crate::model_objects::expressions::SystemExpression;
    use crate::system::save_component::{combine_components, PruningStrategy};
    use crate::tests::reachability::helper_functions::reachability_test_helper_functions;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use test_case::test_case;
    const FOLDER_PATH: &str = "samples/json/EcdarUniversity";

    #[test_case(SystemExpression::Component("Machine".to_string(), None), vec![
        "E0".to_string(),
        "E1".to_string(),
        "E2".to_string(),
        "E3".to_string(),
        "E4".to_string()]; "Simple save component transition id test")]
    #[test_case(
        SystemExpression::Conjunction(
            Box::new(SystemExpression::Component("HalfAdm1".to_string(), None)),
            Box::new(SystemExpression::Component("HalfAdm2".to_string(), None))),
        vec![
            "E0".to_string(),
            "E1".to_string(),
            "E2".to_string(),
            "E3".to_string(),
            "E4".to_string(),
            "E5".to_string(),
            "E6".to_string(),
            "E7".to_string(),
            "E8".to_string(),
            "E9".to_string(),
            "E10".to_string(),
            "E11".to_string()
            ]; "Conjunction save HalfAdm1 and HalfAdm2")]
    fn transition_save_id_checker(
        machine_expression: SystemExpression,
        transition_ids: Vec<String>,
    ) {
        let mock_model = Box::new(machine_expression);
        let mut expected_ids: HashSet<&String> = HashSet::from_iter(transition_ids.iter());
        let (_, system) = reachability_test_helper_functions::create_system_recipe_and_machine(
            *mock_model,
            FOLDER_PATH,
        );

        let mut comp = combine_components(&system, PruningStrategy::NoPruning);

        comp.remake_edge_ids();

        for edge in comp.edges {
            if expected_ids.contains(&edge.id) {
                expected_ids.remove(&edge.id);
            } else {
                panic!("Found unexpected ID in component: {}", &edge.id)
            }
        }
        assert_eq!(expected_ids.len(), 0);
    }
}
