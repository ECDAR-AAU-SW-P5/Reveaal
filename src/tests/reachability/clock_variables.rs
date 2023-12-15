#[cfg(test)]
mod reachability_parser_clock_variable_validation {

    use crate::{
        model_objects::expressions::SystemExpression, system,
        tests::reachability::helper_functions::reachability_test_helper_functions,
    };
    use test_case::test_case;
    const FOLDER_PATH: &str = "samples/json/EcdarUniversity";
    // These tests check that the parser only accepts clock variable arguments with existing clock variables.
    // i.e. check that the variables exist in the model.
    // The model/sample used is samples/json/EcdarUniversity/adm2.json
    // This model/sample contains the clock variables "x" and "y".
    // And locations "L20", "L21" ... "L23".
    #[test_case("Adm2.L20 && Adm2.u>1";
    "The clock variable u in the state does not exist in the model")]
    #[test_case("Adm2.L20 && Adm2.uwu>2";
    "The clock variable uwu in the state does not exist in the model")]
    fn query_parser_checks_invalid_clock_variables(clock_str: &str) {
        let mock_model = SystemExpression::Component("Adm2".to_string(), None);

        let (machine, system) =
            reachability_test_helper_functions::create_system_recipe_and_machine(
                mock_model,
                FOLDER_PATH,
            );

        let mock_state = reachability_test_helper_functions::string_to_state_expr(clock_str);

        assert!(system::extract_state::get_state(&mock_state, &machine, &system).is_err());
    }

    #[test_case("Adm2.L20 && Adm2.x>1";
    "The clock variable x in state exists in the model")]
    #[test_case("Adm2.L20 && Adm2.y<1";
    "The clock variable y in state exists in the model")]
    fn query_parser_checks_valid_clock_variables(clock_str: &str) {
        let mock_model = SystemExpression::Component("Adm2".to_string(), None);
        let (machine, system) =
            reachability_test_helper_functions::create_system_recipe_and_machine(
                mock_model,
                FOLDER_PATH,
            );

        let mock_state = reachability_test_helper_functions::string_to_state_expr(clock_str);

        assert!(system::extract_state::get_state(&mock_state, &machine, &system).is_ok());
    }
}
