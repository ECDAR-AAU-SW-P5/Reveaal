#[cfg(test)]
mod reachability_parser_location_validation {
    use crate::{
        model_objects::expressions::SystemExpression, system,
        tests::reachability::helper_functions::reachability_test_helper_functions,
    };
    use test_case::test_case;
    const FOLDER_PATH: &str = "samples/json/EcdarUniversity";
    // These tests check that the parser only accepts location arguments with existing locations.
    // i.e. check that the locations exist in the model.
    // The model/sample used is samples/json/EcdarUniversity/adm2.json
    // This model/sample contains the locations "L20", "L21" ... "L23".
    #[test_case("Adm2.L19";
    "The location L19 in the state does not exist in the model")]
    #[test_case("Adm2.NOTCORRECTNAME";
    "The location NOTCORRECTNAME in the state does not exist in the model")]
    fn query_parser_checks_invalid_locations(location_str: &str) {
        let mock_model = SystemExpression::Component("Adm2".to_string(), None);
        let (machine, system) =
            reachability_test_helper_functions::create_system_recipe_and_machine(
                mock_model,
                FOLDER_PATH,
            );

        let mock_state = reachability_test_helper_functions::string_to_state_expr(location_str);

        assert!(system::extract_state::get_state(&mock_state, &machine, &system).is_err());
    }

    #[test_case("Adm2.L20";
    "The location L20 in the state exists in the model")]
    #[test_case("Adm2.L23";
    "The location L23 in the state exists in the model")]
    fn query_parser_checks_valid_locations(location_str: &str) {
        let mock_model = SystemExpression::Component("Adm2".to_string(), None);
        let (machine, system) =
            reachability_test_helper_functions::create_system_recipe_and_machine(
                mock_model,
                FOLDER_PATH,
            );

        let mock_state = reachability_test_helper_functions::string_to_state_expr(location_str);

        assert!(system::extract_state::get_state(&mock_state, &machine, &system).is_ok());
    }
}
